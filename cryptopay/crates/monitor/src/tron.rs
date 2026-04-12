use serde::Deserialize;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct TronClient {
    http: Client,
    api_key: String,
    base_url: String,
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
pub struct TransactionInfo {
    pub id: String,
    #[serde(rename = "blockNumber")]
    pub block_number: u64,
    #[serde(rename = "contractRet")]
    pub contract_ret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TronGridResponse {
    data: Vec<Trc20Transfer>,
}

#[derive(Debug, Deserialize)]
struct TransactionInfoResponse {
    data: Vec<TransactionInfo>,
}

#[derive(Debug, Deserialize)]
struct BlockRawData {
    number: u64,
}

#[derive(Debug, Deserialize)]
struct BlockHeader {
    raw_data: BlockRawData,
}

#[derive(Debug, Deserialize)]
struct GetNowBlockResponse {
    block_header: BlockHeader,
}

impl TronClient {
    pub fn new(api_key: String) -> Self {
        let network = std::env::var("TRON_NETWORK").unwrap_or_else(|_| "testnet".to_string());
        let base_url = if network.eq_ignore_ascii_case("mainnet") {
            "https://api.trongrid.io".to_string()
        } else {
            "https://nile.trongrid.io".to_string()
        };

        Self {
            http: Client::new(),
            api_key,
            base_url,
        }
    }

    pub async fn get_trc20_transactions(
        &self,
        address: &str,
        contract: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<Trc20Transfer>> {
        let url = format!("{}/v1/accounts/{}/transactions/trc20", self.base_url, address);

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
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
            return Err(anyhow::anyhow!("TronGrid API error: {} - {}", status, error_body));
        }

        let tr_response: TronGridResponse = response.json().await?;
        Ok(tr_response.data)
    }

    pub async fn get_transaction_info(
        &self,
        tx_hash: &str,
    ) -> anyhow::Result<Option<TransactionInfo>> {
        let url = format!("{}/v1/transactions/{}/info", self.base_url, tx_hash);

        let response = self.http
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
            return Err(anyhow::anyhow!("TronGrid API error: {} - {}", status, error_body));
        }

        let tx_info_response: TransactionInfoResponse = response.json().await?;
        Ok(tx_info_response.data.into_iter().next())
    }

    pub async fn get_current_block(&self) -> anyhow::Result<u64> {
        let url = format!("{}/wallet/getnowblock", self.base_url);

        let response = self.http
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
            return Err(anyhow::anyhow!("TronGrid API error: {} - {}", status, error_body));
        }

        let block_info: GetNowBlockResponse = response.json().await?;
        Ok(block_info.block_header.raw_data.number)
    }
}