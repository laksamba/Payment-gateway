use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_payload(secret: &str, body: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body.as_bytes());
    let result = mac.finalize().into_bytes();
    format!("sha256={}", hex::encode(result))
}

pub fn verify_signature(secret: &str, body: &str, sig: &str) -> bool {
    if !sig.starts_with("sha256=") {
        return false;
    }

    let expected_sig = sign_payload(secret, body);
    // Use constant-time comparison to prevent timing attacks
    constant_time_eq::constant_time_eq(expected_sig.as_bytes(), sig.as_bytes())
}