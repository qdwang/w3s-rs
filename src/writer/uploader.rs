use cid::Cid;
use futures::TryFutureExt;
use reqwest::Client;
use serde::Deserialize;
use std::{fmt, io, str::FromStr};
use thiserror::Error;
use tokio::task::{JoinError, JoinHandle};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Tokio task join error")]
    JoinError(#[from] JoinError),
    #[error("Serde JSON error: {1}")]
    SerdeJSONError(#[source] serde_json::Error, String),
    #[error("Cid parsing error")]
    CidError(#[from] cid::Error),
}

#[derive(Copy, Clone)]
pub enum UploadType {
    Upload,
    Car,
}
impl fmt::Display for UploadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = match self {
            UploadType::Upload => "upload",
            UploadType::Car => "car",
        };
        write!(f, "{}", result)
    }
}

pub struct Uploader {
    upload_type: UploadType,
    auth_token: String,
    name: String,
    tasks: Vec<JoinHandle<Result<Cid, Error>>>,
}

#[derive(Deserialize)]
struct Response {
    cid: String,
}

impl Uploader {
    pub fn new(auth_token: String, name: String, upload_type: UploadType) -> Self {
        Uploader {
            upload_type,
            auth_token,
            name,
            tasks: vec![],
        }
    }

    pub async fn results(self) -> Result<Vec<Cid>, Error> {
        let mut results = Vec::with_capacity(self.tasks.len());
        for task in self.tasks {
            results.push(task.await??);
        }
        Ok(results)
    }

    pub async fn upload(
        name: String,
        upload_type: UploadType,
        auth_token: String,
        content: Vec<u8>,
    ) -> Result<Cid, Error> {
        let api = format!("https://api.web3.storage/{}", upload_type);
        let upload_fn = || {
            Client::new()
                .post(api.clone())
                .header("X-NAME", name.clone())
                .header("accept", "application/json")
                .bearer_auth(auth_token.clone())
                .body(content.clone())
                .send()
                .and_then(|x| x.text())
        };

        let result_str = loop {
            if let Ok(x) = upload_fn().await {
                break x;
            }
        };
        let response: Response =
            serde_json::from_str(&result_str).map_err(|e| Error::SerdeJSONError(e, result_str))?;
        let cid = Cid::from_str(&response.cid)?;
        Ok(cid)
    }
}

impl io::Write for Uploader {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let upload_future = Uploader::upload(
            self.name.clone(),
            self.upload_type,
            self.auth_token.clone(),
            buf.to_vec(),
        );
        let handler = tokio::spawn(upload_future);
        self.tasks.push(handler);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
