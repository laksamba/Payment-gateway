pub mod delivery;
pub mod signing;

#[cfg(test)]
mod tests {
    use super::signing::{sign_payload, verify_signature};

    #[test]
    fn test_webhook_signing() {
        let secret = "test_secret";
        let body = r#"{"event":"payment.confirmed","payment_id":"123"}"#;

        let signature = sign_payload(secret, body);
        assert!(signature.starts_with("sha256="));
        assert!(verify_signature(secret, body, &signature));

        // Test with wrong secret
        assert!(!verify_signature("wrong_secret", body, &signature));

        // Test with wrong body
        let wrong_body = r#"{"event":"payment.failed"}"#;
        assert!(!verify_signature(secret, wrong_body, &signature));
    }
}