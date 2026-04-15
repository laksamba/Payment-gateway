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
    #[serde(alias = "transaction_id", alias = "transactionHash", default)]
    pub transaction_id: String,
    #[serde(alias = "from_address", default)]
    pub from: String,
    #[serde(alias = "to_address", default)]
    pub to: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub token_info: Option<TokenInfo>,
}

#[derive(Debug, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Debug, Deserialize)]
pub struct TransactionInfo {
    #[serde(alias = "txID")]
    pub id: String,
    #[serde(rename = "blockNumber", alias = "block_number", default)]
    pub block_number: u64,
    #[serde(rename = "contractRet", alias = "contract_ret", default)]
    pub contract_ret: Option<String>,
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
        
        // Log API key usage (masked for security)
        let api_key_masked = if self.api_key.len() > 8 {
            format!("{}...{}", &self.api_key[..8], &self.api_key[self.api_key.len()-4..])
        } else {
            "***".to_string()
        };
        tracing::debug!("📡 Querying TRC20 transactions with API key: {} from {}", api_key_masked, self.base_url);

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
            tracing::error!("❌ TronGrid API error on TRC20 query: {} - {}", status, error_body);
            return Err(anyhow::anyhow!("TronGrid API error: {} - {}", status, error_body));
        }

        let text = response.text().await?;
        tracing::debug!("TronGrid TRC20 API response: {}", text);

        let tr_response: serde_json::Value = serde_json::from_str(&text)?;
        
        // Handle different response structures
        let transactions = if let Some(data) = tr_response.get("data").and_then(|v| v.as_array()) {
            data.iter()
                .filter_map(|item| {
                    // Log the raw item to see what fields we get
                    tracing::debug!("Raw TRC20 transfer item: {}", serde_json::to_string_pretty(item).unwrap_or_default());
                    
                    match serde_json::from_value::<Trc20Transfer>(item.clone()) {
                        Ok(tx) => {
                            // Check what transaction_id we got
                            if tx.transaction_id.is_empty() {
                                tracing::warn!("❌ Transaction has empty transaction_id. Item: {:?}", item);
                                None
                            } else {
                                tracing::info!("✓ Parsed TRC20 transfer: tx_id={}, from={}, to={}", 
                                    &tx.transaction_id[..20.min(tx.transaction_id.len())],
                                    &tx.from[..10.min(tx.from.len())],
                                    &tx.to[..10.min(tx.to.len())]
                                );
                                Some(tx)
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse TRC20 transfer: {} - Raw item: {}", e, item);
                            None
                        }
                    }
                })
                .collect()
        } else {
            tracing::warn!("No 'data' array found in TronGrid response");
            Vec::new()
        };

        tracing::info!("Found {} TRC20 transactions for address {}", transactions.len(), address);
        Ok(transactions)
    }

    pub async fn get_transaction_info(
        &self,
        tx_hash: &str,
    ) -> anyhow::Result<Option<TransactionInfo>> {
        // Use wallet endpoint which is more reliable
        let url = format!("{}/wallet/gettransactioninfobyid", self.base_url);
        
        // Log API key usage (masked for security)
        let api_key_masked = if self.api_key.len() > 8 {
            format!("{}...{}", &self.api_key[..8], &self.api_key[self.api_key.len()-4..])
        } else {
            "***".to_string()
        };
        tracing::debug!("📡 Fetching transaction info with API key: {} from {}", api_key_masked, self.base_url);

        let response = self.http
            .post(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "value": tx_hash }))
            .send()
            .await?;

        if response.status().is_success() {
            match response.json::<serde_json::Value>().await {
                Ok(json_value) => {
                    // Log raw response for debugging
                    tracing::debug!("Transaction info response: {}", serde_json::to_string_pretty(&json_value).unwrap_or_default());
                    
                    // Try to extract blockNumber
                    if let Some(block_number) = json_value.get("blockNumber").and_then(|v| v.as_u64()) {
                        // Try multiple locations to find contract status
                        let contract_ret = 
                            // Try ret[0].contractRet
                            json_value.get("ret")
                                .and_then(|r| r.as_array())
                                .and_then(|arr| arr.first())
                                .and_then(|item| item.get("contractRet"))
                                .and_then(|c| c.as_str())
                                .map(|s| s.to_string())
                            // Try top-level contractRet
                            .or_else(|| {
                                json_value.get("contractRet")
                                    .and_then(|c| c.as_str())
                                    .map(|s| s.to_string())
                            })
                            // Try receipt.result (some endpoints use this)
                            .or_else(|| {
                                json_value.get("receipt")
                                    .and_then(|r| r.get("result"))
                                    .and_then(|res| res.as_str())
                                    .map(|s| s.to_string())
                            });
                        
                        tracing::info!("✓ Transaction block={}, contract_ret={:?}", block_number, contract_ret);
                        
                        let tx_info = TransactionInfo {
                            id: tx_hash.to_string(),
                            block_number,
                            contract_ret,
                        };
                        return Ok(Some(tx_info));
                    } else {
                        tracing::debug!("⚠️ No blockNumber in transaction info. Full response: {}", serde_json::to_string_pretty(&json_value).unwrap_or_default());
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to parse transaction info JSON: {}", e);
                }
            }
        } else if response.status().as_u16() == 404 {
            tracing::debug!("⏳ Transaction {} not yet indexed by TronGrid (404 - normal for new txs)", tx_hash);
            return Ok(None);
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
            tracing::error!("❌ TronGrid API error: {} - {}", status, error_body);
            return Err(anyhow::anyhow!("TronGrid API error: {} - {}", status, error_body));
        }
        
        Ok(None)
    }

    pub async fn get_current_block(&self) -> anyhow::Result<u64> {
        let url = format!("{}/wallet/getnowblock", self.base_url);
        
        // Log API key usage (masked for security)
        let api_key_masked = if self.api_key.len() > 8 {
            format!("{}...{}", &self.api_key[..8], &self.api_key[self.api_key.len()-4..])
        } else {
            "***".to_string()
        };
        tracing::debug!("📡 Fetching current block with API key: {} from {}", api_key_masked, self.base_url);

        let response = self.http
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
            tracing::error!("❌ TronGrid API error on block fetch: {} - {}", status, error_body);
            return Err(anyhow::anyhow!("TronGrid API error: {} - {}", status, error_body));
        }

        let block_info: GetNowBlockResponse = response.json().await?;
        Ok(block_info.block_header.raw_data.number)
    }
}
