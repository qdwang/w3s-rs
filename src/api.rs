use std::fmt::Display;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Serde JSON parsing error: {0:?}")]
    SerdeJSONError(#[from] serde_json::Error)
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct StorageItem {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "type")]
    t: String,
    pub name: String,
    created: String,
    updated: String,
    cid: String,
    #[serde(rename = "dagSize")]
    dag_size: u32,
    pins: Vec<Pin>,
    deals: Vec<Deal>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Pin {
    status: String,
    updated: String,
    #[serde(rename = "peerId")]
    peer_id: String,
    #[serde(rename = "peerName")]
    peer_name: String,
    region: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Deal {
    #[serde(rename = "dealId")]
    deal_id: u32,
    #[serde(rename = "storageProvider")]
    storage_provider: String,
    status: String,
    #[serde(rename = "pieceCid")]
    piece_cid: String,
    #[serde(rename = "dataCid")]
    data_cid: String,
    #[serde(rename = "dataModelSelector")]
    data_model_selector: String,
    activation: String,
    created: String,
    updated: String,
}

impl StorageItem {
    pub async fn fetch_uploads(auth_token: impl Display) -> Result<Vec<StorageItem>, Error> {
        let result = Client::new()
            .get("https://api.web3.storage/user/uploads")
            .header("accept", "application/json")
            .bearer_auth(auth_token)
            .send()
            .await?
            .text()
            .await?;

        let items : Vec<StorageItem> = serde_json::from_str(&result)?;
        Ok(items)
    }
    pub fn contains_name(&self, name: &str) -> bool {
        self.name.contains(name)
    }
}
