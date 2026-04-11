pub mod tron;
pub mod scanner;

// USDT TRC-20 contract addresses
pub const USDT_CONTRACT_MAINNET: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
pub const USDT_CONTRACT_TESTNET: &str = "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf";

pub fn get_usdt_contract() -> &'static str {
    match std::env::var("TRON_NETWORK").as_deref() {
        Ok("mainnet") => USDT_CONTRACT_MAINNET,
        _ => USDT_CONTRACT_TESTNET, // default to testnet
    }
}