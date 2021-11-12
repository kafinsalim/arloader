use crate::error::Error;
use crate::transaction::{Base64, DeepHashItem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::{hash::Hash, pubkey::Pubkey, signer::keypair, system_transaction};
use std::str::FromStr;

pub const SOL_AR_PUBKEY: &str = "6AaM5L2SeA7ciwDNaYLhKqQzsDVaQM9CRqXVDdWPeAQ9";
pub const SOL_AR_BASE_URL: &str = "https://arloader.io/";
pub const RATE: u64 = 2500;
pub const FLOOR: u64 = 10000;

pub async fn get_recent_blockhash(base_url: url::Url) -> Result<Hash, Error> {
    let client = reqwest::Client::new();

    let mut config = serde_json::Map::new();
    config.insert(
        "commitment".to_string(),
        Value::String("confirmed".to_string()),
    );

    let post_object = PostObject {
        method: String::from("getRecentBlockhash"),
        ..Default::default()
    };

    let result: Value = client
        .post(base_url)
        .json(&post_object)
        .send()
        .await?
        .json()
        .await?;

    let hash_str = result["result"]["value"]["blockhash"].as_str().unwrap();
    let hash = Hash::from_str(hash_str)?;
    Ok(hash)
}

pub async fn create_sol_transaction(
    base_url: url::Url,
    from_keypair: &keypair::Keypair,
    lamports: u64,
) -> Result<String, Error> {
    let recent_blockhash = get_recent_blockhash(base_url).await?;
    let transaction = system_transaction::transfer(
        from_keypair,
        &Pubkey::from_str(SOL_AR_PUBKEY).unwrap(),
        lamports,
        recent_blockhash,
    );
    let serialized = bincode::serialize(&transaction)?;
    Ok(bs58::encode(serialized).into_string())
}

pub async fn get_sol_ar_signature(
    base_url: url::Url,
    deep_hash_item: DeepHashItem,
    sol_tx: String,
) -> Result<SigResponse, Error> {
    let client = reqwest::Client::new();

    let tx_data = TxData {
        deep_hash_item,
        sol_tx,
    };

    let sig_response: SigResponse = client
        .post(base_url)
        .json(&tx_data)
        .send()
        .await?
        .json()
        .await?;

    Ok(sig_response)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostObject {
    pub jsonrpc: String,
    pub id: usize,
    pub method: String,
    pub params: Vec<Value>,
}

impl Default for PostObject {
    fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getRecentBlockhash".to_string(),
            params: Vec::<Value>::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{create_sol_transaction, get_recent_blockhash};
    use crate::error::Error;
    use solana_sdk::signer::keypair::Keypair;

    #[tokio::test]
    async fn test_get_recent_blockhash() -> Result<(), Error> {
        let base_url = "https://api.devnet.solana.com".parse::<url::Url>().unwrap();

        let result = get_recent_blockhash(base_url).await?;
        println!("{}", result);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_sol_transaction() -> Result<(), Error> {
        let base_url = "https://api.devnet.solana.com".parse::<url::Url>().unwrap();
        let keypair = Keypair::new();

        let result = create_sol_transaction(base_url, &keypair, 42).await?;
        println!("{}", result);
        Ok(())
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct TxData {
    pub deep_hash_item: DeepHashItem,
    pub sol_tx: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SigResponse {
    pub ar_tx_sig: Base64,
    pub ar_tx_id: Base64,
    pub ar_tx_owner: Base64,
    pub sol_tx_sig: String,
    pub lamports: u64,
}