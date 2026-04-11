use std::time::Duration;
use sqlx::PgPool;
use rust_decimal::Decimal;

use crate::tron::TronClient;
use crate::get_usdt_contract;

pub async fn run_scanner(
    pool: PgPool,
    tron: TronClient,
) -> anyhow::Result<()> {
    tracing::info!("Scanner started");

    let usdt_contract = get_usdt_contract();

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
    }
}