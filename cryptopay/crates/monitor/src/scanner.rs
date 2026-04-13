use std::time::Duration;
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::io::Write;
use sqlx::PgPool;
use rust_decimal::Decimal;
use serde_json::json;
use chrono::Utc;

use crate::tron::TronClient;
use crate::get_usdt_contract;

// Direct file logging to bypass buffering
fn log_directly(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("c:\\Users\\sande\\Desktop\\Payment-Gateway\\cryptopay\\scanner_direct.log")
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", timestamp, message);
        let _ = file.flush();
    }
}

fn get_required_confirmations(default: i32) -> i32 {
    std::env::var("REQUIRED_CONFIRMATIONS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default)
}

pub async fn run_scanner(
    pool: PgPool,
    tron: TronClient,
) -> anyhow::Result<()> {
    tracing::info!("🔍 Scanner started and ready to monitor TronGrid for transactions");
    log_directly("✅ SCANNER STARTED AND READY");

    let usdt_contract = get_usdt_contract();
    let mut last_confirmation_check = tokio::time::Instant::now();

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        log_directly("📍 5-second scanner cycle tick");

        // Find all pending payments that haven't expired
        let pending_payments = match cryptopay_db::payments::find_pending_payments(&pool).await {
            Ok(payments) => {
                log_directly(&format!("Found {} pending payments", payments.len()));
                payments
            },
            Err(e) => {
                tracing::error!("Failed to find pending payments: {}", e);
                log_directly(&format!("❌ Error finding pending payments: {}", e));
                continue;
            }
        };

        // First, handle pending payment detection
        if !pending_payments.is_empty() {
            tracing::info!("Scanner cycle: Found {} pending payments - querying TronGrid", pending_payments.len());

            for payment in pending_payments {
                tracing::debug!("Checking payment {}: address={}, amount={}, status={}", 
                    payment.id, payment.deposit_address, payment.amount, payment.status);

                // Get recent TRC20 transactions for this deposit address
                let transactions = match tron.get_trc20_transactions(
                    &payment.deposit_address,
                    usdt_contract,
                    10,
                ).await {
                    Ok(txs) => txs,
                    Err(e) => {
                        tracing::error!("Failed to get TRC20 transactions for {}: {}", payment.deposit_address, e);
                        continue;
                    }
                };

                if transactions.is_empty() {
                    tracing::debug!("No TRC20 transactions found for payment {} (address: {})", payment.id, payment.deposit_address);
                    continue;
                }

                tracing::info!("Checking {} transactions for payment {}", transactions.len(), payment.id);

                // Check for matching transactions
                for (idx, tx) in transactions.iter().enumerate() {
                    tracing::debug!("Transaction {}: id={}, from={}, to={}, value={}", 
                        idx, 
                        if tx.transaction_id.is_empty() { "EMPTY" } else { &tx.transaction_id[..20.min(tx.transaction_id.len())] },
                        &tx.from[..10.min(tx.from.len())],
                        &tx.to[..10.min(tx.to.len())],
                        tx.value
                    );

                    // Validate transaction_id is not empty
                    if tx.transaction_id.is_empty() {
                        tracing::warn!("Skipping transaction {} with empty transaction_id for payment {}", idx, payment.id);
                        continue;
                    }

                    // Case-insensitive address comparison
                    let tx_to_lower = tx.to.to_lowercase();
                    let deposit_addr_lower = payment.deposit_address.to_lowercase();

                    if tx_to_lower != deposit_addr_lower {
                        tracing::debug!("Transaction {} address mismatch: tx.to='{}' != deposit_address='{}'", 
                            idx, tx.to, payment.deposit_address);
                        continue;
                    }

                    tracing::info!("Address match for payment {}! Processing transaction...", payment.id);

                    // Parse the value (USDT has 6 decimals)
                    let tx_value = match tx.value.parse::<u128>() {
                        Ok(v) => Decimal::from(v) / Decimal::from(1_000_000),
                        Err(e) => {
                            tracing::error!("Failed to parse transaction value '{}': {}", tx.value, e);
                            continue;
                        }
                    };

                    tracing::info!("Transaction amount check: receipt={} vs required={}", tx_value, payment.amount);

                    // Check if the transaction amount meets or exceeds the payment amount
                    if tx_value >= payment.amount {
                        tracing::info!("✓ Amount MATCH! Attempting to mark payment {} as detected with tx_hash: {}", payment.id, tx.transaction_id);
                        
                        // Mark payment as detected
                        match cryptopay_db::payments::mark_payment_detected(
                            &pool,
                            payment.id,
                            &tx.transaction_id,
                        ).await {
                            Ok(_) => {
                                tracing::info!("✓✓ Payment {} successfully marked as detected!", payment.id);
                            },
                            Err(e) => {
                                tracing::error!("✗ FAILED to mark payment {} as detected: {}", payment.id, e);
                                tracing::error!("  This usually means the payment status is no longer 'pending'");
                                tracing::error!("  Check if the payment was already updated or if there's a DB constraint issue");
                                continue;
                            }
                        }

                        break; // Stop checking other transactions for this payment
                    } else {
                        tracing::warn!("Transaction value {} < payment amount {} for payment {}", tx_value, payment.amount, payment.id);
                    }
                }
            }
        } else {
            tracing::debug!("Scanner cycle: No pending payments to check");
        }

        // Now check confirmation status (this runs EVERY 15 seconds regardless of pending payments)
        if last_confirmation_check.elapsed() >= Duration::from_secs(15) {
            last_confirmation_check = tokio::time::Instant::now();
            
            tracing::info!("⏰ Confirmation check cycle started...");
            log_directly("🔄 ==== CONFIRMATION CHECK CYCLE STARTED ====");

            let detected_payments = match cryptopay_db::payments::find_detected_payments(&pool).await {
                Ok(payments) => {
                    if payments.is_empty() {
                        tracing::debug!("No detected/confirming payments to check confirmations for");
                        log_directly("ℹ️ No detected/confirming payments found");
                        continue;
                    }
                    log_directly(&format!("✓ Found {} detected/confirming payments", payments.len()));
                    payments
                },
                Err(e) => {
                    tracing::error!("Failed to find detected/confirming payments: {}", e);
                    log_directly(&format!("❌ Error finding detected/confirming payments: {}", e));
                    continue;
                }
            };

            tracing::info!("Checking confirmations for {} payments", detected_payments.len());

            let current_block = match tron.get_current_block().await {
                Ok(block) => block,
                Err(e) => {
                    tracing::error!("❌ Failed to fetch current Tron block: {}", e);
                    continue;
                }
            };
            
            tracing::info!("✓ Current Tron block: {}", current_block);

            for payment in detected_payments {
                let tx_hash = match payment.tx_hash.as_deref() {
                    Some(hash) => hash,
                    None => {
                        tracing::warn!("⚠️ Payment {} has no tx_hash, skipping confirmation check", payment.id);
                        log_directly(&format!("⚠️ Payment {} has no tx_hash", payment.id));
                        continue;
                    }
                };

                tracing::info!("🔗 Fetching confirmation details for tx_hash: {}", tx_hash);
                log_directly(&format!("🔍 Fetching tx info for payment: {}", payment.id));

                let tx_info = match tron.get_transaction_info(tx_hash).await {
                    Ok(Some(info)) => {
                        tracing::info!("✓ Transaction info received: block_number={}, contract_ret={}", info.block_number, info.contract_ret.as_deref().unwrap_or("?"));
                        log_directly(&format!("✓ Got tx_info: tx_block={}, current_block={}", info.block_number, current_block));
                        info
                    },
                    Ok(None) => {
                        tracing::warn!("⚠️ Transaction info response empty for {}", tx_hash);
                        log_directly(&format!("⚠️ Empty tx_info for payment {}", payment.id));
                        continue;
                    },
                    Err(e) => {
                        let error_msg = e.to_string();
                        if error_msg.contains("404") {
                            tracing::debug!("⏳ Transaction {} not yet indexed by TronGrid (404 - normal for new txs), will retry", tx_hash);
                            log_directly(&format!("⏳ Payment {} not yet indexed (404)", payment.id));
                        } else {
                            tracing::error!("❌ Failed to fetch transaction info for {}: {}", tx_hash, e);
                            log_directly(&format!("❌ Error fetching tx_info: {}", e));
                        }
                        continue;
                    }
                };

                let confirmations = current_block.saturating_sub(tx_info.block_number) as i32;
                let required_confirmations = get_required_confirmations(payment.required_confirmations);
                log_directly(&format!("📊 Payment {}: confirmations={} vs required={}", payment.id, confirmations, required_confirmations));

                tracing::info!("📊 Confirmation status: block_height={}, tx_block={}, confirmations={}/{}", 
                    current_block, tx_info.block_number, confirmations, required_confirmations);

                let new_status = if tx_info.contract_ret.as_deref() != Some("SUCCESS") {
                    "failed"
                } else if confirmations >= required_confirmations {
                    "confirmed"
                } else if confirmations > 0 {
                    "confirming"
                } else {
                    payment.status.as_str()
                };

                tracing::info!("Status transition: {} → {}", payment.status, new_status);
                log_directly(&format!("🔄 Status change: {} → {}", payment.status, new_status));

                if new_status == "confirmed" {
                    log_directly(&format!("✅ CONFIRMED! Payment {} reached {} confirmations", payment.id, confirmations));
                    if let Err(e) = cryptopay_db::payments::mark_payment_confirmed(
                        &pool,
                        payment.id,
                        confirmations,
                    ).await {
                        tracing::error!("Failed to mark payment {} as confirmed: {}", payment.id, e);
                        log_directly(&format!("❌ Error marking as confirmed: {}", e));
                        continue;
                    }

                    if let Err(e) = cryptopay_db::events::insert_event(
                        &pool,
                        payment.id,
                        "payment.confirmed",
                        json!({
                            "tx_hash": tx_hash,
                            "confirmations": confirmations,
                            "amount": payment.amount,
                        }),
                    ).await {
                        tracing::error!("Failed to insert confirmed event for {}: {}", payment.id, e);
                        log_directly(&format!("⚠️ Could not insert confirmed event: {}", e));
                    }

                    tracing::info!("Payment confirmed: {}", payment.id);

                    // Queue webhook if merchant has webhook_url configured
                    if let Some(merchant) = cryptopay_db::merchants::find_merchant_by_id(&pool, payment.merchant_id).await? {
                        if let Some(webhook_url) = &merchant.webhook_url {
                            let webhook_payload = cryptopay_webhook::delivery::WebhookPayload {
                                event: "payment.confirmed".to_string(),
                                payment_id: payment.id,
                                tx_hash: payment.tx_hash.clone(),
                                amount: payment.amount.to_string(),
                                currency: payment.currency.clone(),
                                confirmed_at: Some(Utc::now()),
                                metadata: payment.metadata.clone(),
                            };

                            if let Err(e) = cryptopay_webhook::delivery::WebhookDelivery::queue_webhook(
                                &pool,
                                payment.id,
                                "payment.confirmed",
                                webhook_url,
                                &webhook_payload,
                            ).await {
                                tracing::error!("Failed to queue webhook for payment {}: {}", payment.id, e);
                            } else {
                                tracing::info!("Webhook queued for payment {}", payment.id);
                            }
                        }
                    }
                } else if new_status == "confirming" && payment.status != "confirming" {
                    log_directly(&format!("📢 Updating payment {} to 'confirming' status with {} confirmations", payment.id, confirmations));
                    // Update confirmations count and mark as confirming
                    if let Err(e) = cryptopay_db::payments::update_payment_confirmations(
                        &pool,
                        payment.id,
                        confirmations,
                        "confirming",
                    ).await {
                        tracing::error!("Failed to update payment {} confirmations: {}", payment.id, e);
                        continue;
                    }

                    tracing::info!("Payment confirming: {} ({} confirmations)", payment.id, confirmations);
                } else {
                    if let Err(e) = cryptopay_db::payments::update_payment_confirmations(
                        &pool,
                        payment.id,
                        confirmations,
                        new_status,
                    ).await {
                        tracing::error!("Failed to update confirmations for {}: {}", payment.id, e);
                    }
                }
            }
        }
    }
}