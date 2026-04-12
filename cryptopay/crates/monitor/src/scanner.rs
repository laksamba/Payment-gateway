use std::time::Duration;
use sqlx::PgPool;
use rust_decimal::Decimal;
use serde_json::json;
use chrono::Utc;

use crate::tron::TronClient;
use crate::get_usdt_contract;

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
    tracing::info!("Scanner started");

    let usdt_contract = get_usdt_contract();
    let mut last_confirmation_check = tokio::time::Instant::now();

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Find all pending payments that haven't expired
        let pending_payments = match cryptopay_db::payments::find_pending_payments(&pool).await {
            Ok(payments) => payments,
            Err(e) => {
                tracing::error!("Failed to find pending payments: {}", e);
                continue;
            }
        };

        for payment in pending_payments {
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

            // Check for matching transactions
            for tx in transactions {
                if tx.to == payment.deposit_address {
                    // Parse the value (USDT has 6 decimals)
                    let tx_value = match tx.value.parse::<u128>() {
                        Ok(v) => Decimal::from(v) / Decimal::from(1_000_000),
                        Err(e) => {
                            tracing::error!("Failed to parse transaction value '{}': {}", tx.value, e);
                            continue;
                        }
                    };

                    // Check if the transaction amount meets or exceeds the payment amount
                    if tx_value >= payment.amount {
                        // Mark payment as detected
                        if let Err(e) = cryptopay_db::payments::mark_payment_detected(
                            &pool,
                            payment.id,
                            &tx.transaction_id,
                        ).await {
                            tracing::error!("Failed to mark payment {} as detected: {}", payment.id, e);
                            continue;
                        }

                        tracing::info!("Payment detected: {}", payment.id);
                        break; // Stop checking other transactions for this payment
                    }
                }
            }
        }

        if last_confirmation_check.elapsed() >= Duration::from_secs(15) {
            last_confirmation_check = tokio::time::Instant::now();

            let detected_payments = match cryptopay_db::payments::find_detected_payments(&pool).await {
                Ok(payments) => payments,
                Err(e) => {
                    tracing::error!("Failed to find detected/confirming payments: {}", e);
                    continue;
                }
            };

            let current_block = match tron.get_current_block().await {
                Ok(block) => block,
                Err(e) => {
                    tracing::error!("Failed to fetch current Tron block: {}", e);
                    continue;
                }
            };

            for payment in detected_payments {
                let tx_hash = match payment.tx_hash.as_deref() {
                    Some(hash) => hash,
                    None => continue,
                };

                let tx_info = match tron.get_transaction_info(tx_hash).await {
                    Ok(Some(info)) => info,
                    Ok(None) => continue,
                    Err(e) => {
                        tracing::error!("Failed to fetch transaction info for {}: {}", tx_hash, e);
                        continue;
                    }
                };

                let confirmations = current_block.saturating_sub(tx_info.block_number) as i32;
                let required_confirmations = get_required_confirmations(payment.required_confirmations);

                let new_status = if tx_info.contract_ret.as_deref() != Some("SUCCESS") {
                    "failed"
                } else if confirmations >= required_confirmations {
                    "confirmed"
                } else if confirmations > 0 {
                    "confirming"
                } else {
                    payment.status.as_str()
                };

                if new_status == "confirmed" {
                    if let Err(e) = cryptopay_db::payments::mark_payment_confirmed(
                        &pool,
                        payment.id,
                        confirmations,
                    ).await {
                        tracing::error!("Failed to mark payment {} as confirmed: {}", payment.id, e);
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