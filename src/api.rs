use std::fmt::Display;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Serde JSON parsing error. Response: {1}")]
    SerdeJSONError(#[source] serde_json::Error, String),
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
    pub cid: String,
    #[serde(rename = "dagSize")]
    dag_size: u32,
    pins: Vec<Pin>,
    deals: Vec<Deal>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Status {
    created: String,
    pub cid: String,
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

#[derive(Debug)]
pub enum UserUploadsSortBy {
    Date,
    Name,
}
impl Display for UserUploadsSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum UserUploadsSortOrder {
    Asc,
    Desc,
}
impl Display for UserUploadsSortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct UserUploadsQuery {
    before: String,
    sort_by: UserUploadsSortBy,
    sort_order: UserUploadsSortOrder,
    page: u32,
    size: u32,
}
impl AsRef<UserUploadsQuery> for UserUploadsQuery {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl UserUploadsQuery {
    pub fn new(
        page: Option<u32>,
        size: Option<u32>,
        sort_by: Option<UserUploadsSortBy>,
        sort_order: Option<UserUploadsSortOrder>,
        before: Option<String>,
    ) -> Self {
        UserUploadsQuery {
            before: before.unwrap_or("3000-01-01T00:00:00Z".to_string()),
            sort_by: sort_by.unwrap_or(UserUploadsSortBy::Date),
            sort_order: sort_order.unwrap_or(UserUploadsSortOrder::Desc),
            page: page.unwrap_or(1),
            size: size.unwrap_or(100),
        }
    }
    fn gen_query(&self) -> Vec<(&'static str, String)> {
        let mut ret = vec![];

        ret.push(("before", self.before.clone()));
        ret.push(("sortBy", self.sort_by.to_string()));
        ret.push(("sortOrder", self.sort_order.to_string()));
        ret.push(("page", self.page.to_string()));
        ret.push(("size", self.size.to_string()));
        ret
    }
}

impl StorageItem {
    pub async fn fetch_uploads(
        auth_token: impl Display,
        query: impl AsRef<UserUploadsQuery>,
    ) -> Result<Vec<StorageItem>, Error> {
        let result = Client::new()
            .get("https://api.web3.storage/user/uploads")
            .header("accept", "application/json")
            .query(&query.as_ref().gen_query())
            .bearer_auth(auth_token)
            .send()
            .await?
            .text()
            .await?;

        let items: Vec<StorageItem> =
            serde_json::from_str(&result).map_err(|e| Error::SerdeJSONError(e, result))?;

        Ok(items)
    }

    pub async fn retrieve_car(cid: &str) -> Result<Vec<u8>, Error> {
        let result = Client::new()
            .get(format!("https://api.web3.storage/car/{}", cid))
            .header("accept", "application/vnd.ipld.car")
            .send()
            .await?
            .bytes()
            .await?;

        Ok(result.to_vec())
    }

    pub async fn check_car_head(cid: &str) -> Result<String, Error> {
        let result = Client::new()
            .head(format!("https://api.web3.storage/car/{}", cid))
            .header("accept", "*/*")
            .send()
            .await?
            .text()
            .await?;

        Ok(result)
    }

    pub async fn status_of_cid(cid: &str) -> Result<Status, Error> {
        let result = Client::new()
            .get(format!("https://api.web3.storage/status/{}", cid))
            .header("accept", "application/json")
            .send()
            .await?
            .text()
            .await?;

        let status: Status =
            serde_json::from_str(&result).map_err(|e| Error::SerdeJSONError(e, result))?;

        Ok(status)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.name.contains(name)
    }
}
