use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct TronClient {
    http: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct Trc20Transfer {
    pub transaction_id: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub token_info: TokenInfo,
}

#[derive(Debug, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Debug, Deserialize)]
struct TronGridResponse {
    data: Vec<Trc20Transfer>,
}

impl TronClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: Client::new(),
            api_key,
        }
    }

    pub async fn get_trc20_transactions(
        &self,
        address: &str,
        contract: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<Trc20Transfer>> {
        let url = format!("https://api.trongrid.io/v1/accounts/{}/transactions/trc20", address);

        let response = self.http
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .query(&[
                ("contract_address", contract),
                ("limit", &limit.to_string()),
                ("only_confirmed", "false"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("TronGrid API error: {}", response.status()));
        }

        let tr_response: TronGridResponse = response.json().await?;
        Ok(tr_response.data)
    }
}