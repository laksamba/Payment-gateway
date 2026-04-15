use rust_decimal::Decimal;

use crate::errors::AppError;

pub fn parse_amount(s: &str) -> Result<Decimal, AppError> {
    let amount: Decimal = s.parse()
        .map_err(|_| AppError::BadRequest(format!("invalid amount: {}", s)))?;

    if amount < Decimal::ZERO {
        return Err(AppError::BadRequest("amount cannot be negative".to_string()));
    }

    if amount == Decimal::ZERO {
        return Err(AppError::BadRequest("amount cannot be zero".to_string()));
    }

    // Check decimal places (scale)
    let mut temp = amount;
    let mut places = 0;
    while temp.fract() != Decimal::ZERO {
        temp *= Decimal::from(10);
        places += 1;
        if places > 6 {
            return Err(AppError::BadRequest("amount cannot have more than 6 decimal places".to_string()));
        }
    }

    // Max 1_000_000 USDT
    let max = Decimal::from(1_000_000);
    if amount > max {
        return Err(AppError::BadRequest("amount exceeds maximum of 1_000_000 USDT".to_string()));
    }

    Ok(amount)
}
