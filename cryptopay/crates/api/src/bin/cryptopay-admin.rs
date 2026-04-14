use anyhow::{anyhow, bail, Context, Result};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

enum Command {
    ExportPaymentKey { payment_id: Uuid },
    ListSweepCandidates { limit: i64 },
    MarkPaymentSwept { payment_id: Uuid, sweep_tx_hash: String },
    MarkPaymentSweepFailed { payment_id: Uuid, reason: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    match parse_args()? {
        Command::ExportPaymentKey { payment_id } => export_payment_key(payment_id).await,
        Command::ListSweepCandidates { limit } => list_sweep_candidates(limit).await,
        Command::MarkPaymentSwept {
            payment_id,
            sweep_tx_hash,
        } => mark_payment_swept(payment_id, &sweep_tx_hash).await,
        Command::MarkPaymentSweepFailed { payment_id, reason } => {
            mark_payment_sweep_failed(payment_id, &reason).await
        }
    }
}

fn parse_args() -> Result<Command> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.as_slice() {
        [] => {
            print_usage();
            std::process::exit(0);
        }
        [flag] if flag == "--help" || flag == "-h" => {
            print_usage();
            std::process::exit(0);
        }
        [command, flag, payment_id] if command == "export-payment-key" && flag == "--payment-id" => {
            Ok(Command::ExportPaymentKey {
                payment_id: parse_uuid_arg(payment_id, "payment id")?,
            })
        }
        [command] if command == "list-sweep-candidates" => {
            Ok(Command::ListSweepCandidates { limit: 50 })
        }
        [command, flag, limit] if command == "list-sweep-candidates" && flag == "--limit" => {
            Ok(Command::ListSweepCandidates {
                limit: limit
                    .parse::<i64>()
                    .with_context(|| format!("invalid limit: {}", limit))?,
            })
        }
        [command, payment_flag, payment_id, tx_flag, sweep_tx_hash]
            if command == "mark-payment-swept"
                && payment_flag == "--payment-id"
                && tx_flag == "--sweep-tx-hash" =>
        {
            Ok(Command::MarkPaymentSwept {
                payment_id: parse_uuid_arg(payment_id, "payment id")?,
                sweep_tx_hash: sweep_tx_hash.to_string(),
            })
        }
        [command, payment_flag, payment_id, reason_flag, reason]
            if command == "mark-payment-sweep-failed"
                && payment_flag == "--payment-id"
                && reason_flag == "--reason" =>
        {
            Ok(Command::MarkPaymentSweepFailed {
                payment_id: parse_uuid_arg(payment_id, "payment id")?,
                reason: reason.to_string(),
            })
        }
        _ => {
            print_usage();
            Err(anyhow!("invalid arguments"))
        }
    }
}

async fn export_payment_key(payment_id: Uuid) -> Result<()> {
    let db = connect_db().await?;
    let master_wallet_private_key = std::env::var("MASTER_WALLET_PRIVATE_KEY")
        .context("MASTER_WALLET_PRIVATE_KEY must be set for admin commands")?;

    let payment = cryptopay_db::payments::find_payment_by_id(&db, payment_id)
        .await?
        .ok_or_else(|| anyhow!("payment {} not found", payment_id))?;

    let derived_wallet =
        cryptopay_db::payments::derive_deposit_wallet(&master_wallet_private_key, payment.id)?;

    if derived_wallet.deposit_address != payment.deposit_address {
        bail!(
            "derived address {} does not match stored payment address {} for payment {}",
            derived_wallet.deposit_address,
            payment.deposit_address,
            payment.id
        );
    }

    let merchant = cryptopay_db::merchants::find_merchant_by_id(&db, payment.merchant_id)
        .await?
        .ok_or_else(|| anyhow!("merchant {} not found", payment.merchant_id))?;

    let output = json!({
        "payment_id": payment.id,
        "merchant_id": payment.merchant_id,
        "merchant_email": merchant.email,
        "deposit_address": payment.deposit_address,
        "private_key_hex": derived_wallet.private_key_hex,
        "status": payment.status,
        "amount": payment.amount.to_string(),
        "currency": payment.currency,
        "tx_hash": payment.tx_hash,
        "confirmations": payment.confirmations,
        "required_confirmations": payment.required_confirmations,
        "created_at": payment.created_at.to_rfc3339(),
        "confirmed_at": payment.confirmed_at.map(|value| value.to_rfc3339()),
        "sweep_status": payment.sweep_status,
        "sweep_tx_hash": payment.sweep_tx_hash,
        "swept_at": payment.swept_at.map(|value| value.to_rfc3339()),
        "sweep_error": payment.sweep_error,
        "metadata": payment.metadata,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn list_sweep_candidates(limit: i64) -> Result<()> {
    let db = connect_db().await?;
    let master_wallet_private_key = std::env::var("MASTER_WALLET_PRIVATE_KEY").ok();
    let payments = cryptopay_db::payments::list_sweep_candidates(&db, limit).await?;

    let mut output = Vec::with_capacity(payments.len());
    for payment in payments {
        let merchant = cryptopay_db::merchants::find_merchant_by_id(&db, payment.merchant_id)
            .await?
            .ok_or_else(|| anyhow!("merchant {} not found", payment.merchant_id))?;
        let exportable_with_current_master_key = master_wallet_private_key
            .as_deref()
            .map(|key| {
                cryptopay_db::payments::derive_deposit_wallet(key, payment.id)
                    .map(|wallet| wallet.deposit_address == payment.deposit_address)
                    .unwrap_or(false)
            });

        output.push(json!({
            "payment_id": payment.id,
            "merchant_id": payment.merchant_id,
            "merchant_email": merchant.email,
            "deposit_address": payment.deposit_address,
            "amount": payment.amount.to_string(),
            "currency": payment.currency,
            "status": payment.status,
            "tx_hash": payment.tx_hash,
            "confirmations": payment.confirmations,
            "confirmed_at": payment.confirmed_at.map(|value| value.to_rfc3339()),
            "sweep_status": payment.sweep_status,
            "sweep_error": payment.sweep_error,
            "exportable_with_current_master_key": exportable_with_current_master_key,
        }));
    }

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn mark_payment_swept(payment_id: Uuid, sweep_tx_hash: &str) -> Result<()> {
    if sweep_tx_hash.trim().is_empty() {
        bail!("sweep_tx_hash must not be empty");
    }

    let db = connect_db().await?;
    let payment = cryptopay_db::payments::mark_payment_swept(&db, payment_id, sweep_tx_hash).await?;

    println!("{}", serde_json::to_string_pretty(&sweep_status_json(&payment))?);
    Ok(())
}

async fn mark_payment_sweep_failed(payment_id: Uuid, reason: &str) -> Result<()> {
    if reason.trim().is_empty() {
        bail!("reason must not be empty");
    }

    let db = connect_db().await?;
    let payment = cryptopay_db::payments::mark_payment_sweep_failed(&db, payment_id, reason).await?;

    println!("{}", serde_json::to_string_pretty(&sweep_status_json(&payment))?);
    Ok(())
}

async fn connect_db() -> Result<PgPool> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL must be set for admin commands")?;
    let db = cryptopay_db::connect(&database_url).await?;
    Ok(db)
}

fn parse_uuid_arg(value: &str, label: &str) -> Result<Uuid> {
    Uuid::parse_str(value).with_context(|| format!("invalid {}: {}", label, value))
}

fn sweep_status_json(payment: &cryptopay_core::Payment) -> serde_json::Value {
    json!({
        "payment_id": payment.id,
        "status": payment.status,
        "sweep_status": payment.sweep_status,
        "sweep_tx_hash": payment.sweep_tx_hash,
        "swept_at": payment.swept_at.map(|value| value.to_rfc3339()),
        "sweep_error": payment.sweep_error,
    })
}

fn print_usage() {
    eprintln!(
        "Usage:\n  \
cryptopay-admin export-payment-key --payment-id <uuid>\n  \
cryptopay-admin list-sweep-candidates [--limit <count>]\n  \
cryptopay-admin mark-payment-swept --payment-id <uuid> --sweep-tx-hash <hash>\n  \
cryptopay-admin mark-payment-sweep-failed --payment-id <uuid> --reason <text>\n\n\
These are local admin-only commands for exporting deposit keys and tracking sweep state."
    );
}
