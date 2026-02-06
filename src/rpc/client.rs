use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

const DEFAULT_RPC_URL: &str = "127.0.0.1:16210";
pub const PUBLIC_TESTNET10_GRPC: &str = "https://api-tn10.kaspa.org:16110";
pub const PUBLIC_TESTNET10_RPC: &str = "https://api-tn10.kaspa.org";

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("JSON error: {0}")]
    JsonError(String),
    #[error("Invalid response")]
    InvalidResponse,
}

pub struct RpcClient {
    url: String,
    client: reqwest::Client,
}

impl RpcClient {
    pub fn new(rpc_url: Option<&str>) -> Self {
        let url = rpc_url.unwrap_or(PUBLIC_TESTNET10_RPC).trim_end_matches('/').to_string();
        Self {
            url,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    fn build_client(&self) -> Result<&reqwest::Client, RpcError> {
        Ok(&self.client)
    }

    pub async fn get_balance_by_address(&self, address: &str) -> Result<GetBalanceByAddressResponse, RpcError> {
        let client = self.build_client()?;
        
        let url = format!("{}/addresses/{}/balance", self.url, address);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RpcError::Rpc(format!("HTTP {}: {}", status, text)));
        }

        let balance_response: RestBalanceResponse = response
            .json()
            .await
            .map_err(|e| RpcError::JsonError(e.to_string()))?;

        Ok(GetBalanceByAddressResponse {
            balance: balance_response.balance,
        })
    }

    pub async fn get_utxos_by_address(&self, address: &str) -> Result<GetUtxosByAddressResponse, RpcError> {
        let client = self.build_client()?;

        let url = format!("{}/addresses/{}/utxos", self.url, address);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RpcError::Rpc(format!("HTTP {}: {}", status, text)));
        }

        let utxos_response: RestUtxosResponse = response
            .json()
            .await
            .map_err(|e| RpcError::JsonError(e.to_string()))?;

        let entries: Vec<GetUtxosByAddressEntry> = utxos_response.entries.into_iter().map(|e| {
            GetUtxosByAddressEntry {
                address: e.address,
                outpoint: GetOutPoint {
                    transaction_id: e.outpoint.transaction_id,
                    index: e.outpoint.index,
                },
                utxo_entry: GetUtxoEntry {
                    amount: e.amount,
                    script_public_key: GetScriptPublicKey {
                        version: e.script_public_key.version,
                        script: e.script_public_key.script,
                    },
                    block_daa_score: e.block_daa_score,
                    is_coinbase: e.is_coinbase,
                },
                is_spent: e.is_spent,
            }
        }).collect();

        Ok(GetUtxosByAddressResponse { entries })
    }

    pub async fn get_utxos_by_addresses(&self, addresses: Vec<String>) -> Result<GetUtxosByAddressesResponse, RpcError> {
        let client = self.build_client()?;

        let url = format!("{}/addresses/utxos", self.url);

        let body = serde_json::json!({
            "addresses": addresses
        });

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RpcError::Rpc(format!("HTTP {}: {}", status, text)));
        }

        let utxos_response: RestUtxosResponse = response
            .json()
            .await
            .map_err(|e| RpcError::JsonError(e.to_string()))?;

        let entries: Vec<GetUtxosByAddressesEntry> = utxos_response.entries.into_iter().map(|e| {
            GetUtxosByAddressesEntry {
                address: e.address,
                outpoint: GetOutPoint {
                    transaction_id: e.outpoint.transaction_id,
                    index: e.outpoint.index,
                },
                utxo_entry: GetUtxoEntry {
                    amount: e.amount,
                    script_public_key: GetScriptPublicKey {
                        version: e.script_public_key.version,
                        script: e.script_public_key.script,
                    },
                    block_daa_score: e.block_daa_score,
                    is_coinbase: e.is_coinbase,
                },
                is_spent: e.is_spent,
            }
        }).collect();

        Ok(GetUtxosByAddressesResponse { entries })
    }

    pub async fn submit_transaction(
        &self,
        tx_json: &serde_json::Value,
    ) -> Result<SubmitTransactionResponse, RpcError> {
        let client = self.build_client()?;
        
        // Try sending transaction as raw JSON (not wrapped)
        let url = format!("{}/transactions", self.url);
        
        let body = serde_json::json!({
            "transaction": tx_json,
            "allowOrphan": false
        });

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RpcError::Rpc(format!("HTTP {}: {}", status, text)));
        }

        let submit_response: RestSubmitResponse = response
            .json()
            .await
            .map_err(|e| RpcError::JsonError(e.to_string()))?;

        Ok(SubmitTransactionResponse {
            transaction_id: submit_response.transaction_id,
        })
    }

    pub async fn submit_transaction_hex(
        &self,
        tx_hex: &str,
    ) -> Result<SubmitTransactionResponse, RpcError> {
        let client = self.build_client()?;
        
        let url = format!("{}/transactions", self.url);
        
        let body = serde_json::json!({
            "transaction": tx_hex,
            "allowOrphan": false
        });

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RpcError::Rpc(format!("HTTP {}: {}", status, text)));
        }

        let submit_response: RestSubmitResponse = response
            .json()
            .await
            .map_err(|e| RpcError::JsonError(e.to_string()))?;

        Ok(SubmitTransactionResponse {
            transaction_id: submit_response.transaction_id,
        })
    }
}

// REST API response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestBalanceResponse {
    pub address: String,
    #[serde(deserialize_with = "deserialize_string_or_u64")]
    pub balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestUtxoEntry {
    pub address: String,
    pub outpoint: RestOutPoint,
    pub amount: u64,
    pub script_public_key: RestScriptPublicKey,
    pub block_daa_score: u64,
    pub is_coinbase: bool,
    pub is_spent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestOutPoint {
    pub transaction_id: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestScriptPublicKey {
    pub version: u16,
    pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestUtxosResponse {
    pub entries: Vec<RestUtxoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestSubmitResponse {
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBalanceByAddressResponse {
    pub balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUtxosByAddressResponse {
    pub entries: Vec<GetUtxosByAddressEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUtxosByAddressesResponse {
    pub entries: Vec<GetUtxosByAddressesEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOutPoint {
    pub transaction_id: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetScriptPublicKey {
    pub version: u16,
    pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUtxoEntry {
    pub amount: u64,
    pub script_public_key: GetScriptPublicKey,
    pub block_daa_score: u64,
    pub is_coinbase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUtxosByAddressEntry {
    pub address: String,
    pub outpoint: GetOutPoint,
    pub utxo_entry: GetUtxoEntry,
    pub is_spent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUtxosByAddressesEntry {
    pub address: String,
    pub outpoint: GetOutPoint,
    pub utxo_entry: GetUtxoEntry,
    pub is_spent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub transaction_id: String,
}

fn deserialize_string_or_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map_or_else(
        |e| Err(e),
        |value| match value {
            serde_json::Value::Number(n) => {
                if let Some(n) = n.as_u64() {
                    Ok(n)
                } else if let Some(n) = n.as_f64() {
                    Ok(n as u64)
                } else {
                    Err(serde::de::Error::custom("Invalid number"))
                }
            }
            serde_json::Value::String(s) => {
                s.parse().map_err(|_| serde::de::Error::custom("Invalid string"))
            }
            _ => Err(serde::de::Error::custom("Invalid type")),
        },
    )
}
