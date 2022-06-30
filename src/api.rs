use serde::Deserialize;

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
    pub fn contains_name(&self, name: &str) -> bool {
        self.name.contains(name)
    }
}