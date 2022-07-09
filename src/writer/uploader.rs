use cid::Cid;
use core::task::Poll;
use futures::TryFutureExt;
use reqwest::{Body, Client};
use serde::Deserialize;
use std::{
    cmp, fmt, io, mem,
    str::FromStr,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tokio::{
    runtime::Handle,
    task::{JoinError, JoinHandle},
};

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

type ProgressListener =
    Arc<Mutex<dyn FnMut(Arc<String>, usize, usize, usize) + Send + Sync + 'static>>;
pub struct Uploader {
    upload_type: UploadType,
    auth_token: Arc<String>,
    name: Arc<String>,
    max_connection: usize,
    tasks: Vec<JoinHandle<Result<Cid, Error>>>,
    results: Vec<Cid>,
    progress_listener: Option<ProgressListener>,
}

impl Uploader {
    pub fn new(
        auth_token: String,
        name: String,
        upload_type: UploadType,
        max_connection: usize,
        progress_listener: Option<ProgressListener>,
    ) -> Self {
        Uploader {
            upload_type,
            auth_token: Arc::new(auth_token),
            name: Arc::new(name),
            max_connection,
            tasks: vec![],
            results: vec![],
            progress_listener,
        }
    }

    pub fn pause_to_complete_tasks(&mut self) {
        if self.tasks.len() == self.max_connection {
            tokio::task::block_in_place(|| {
                let result = Handle::current().block_on(self.finish_results(false));
                if let Ok(cid_vec) = result {
                    self.results.extend(cid_vec);
                }
            });
        }
    }

    pub async fn finish_results(&mut self, with_prev: bool) -> Result<Vec<Cid>, Error> {
        let tasks = mem::replace(&mut self.tasks, vec![]);

        let mut results = if with_prev {
            mem::replace(&mut self.results, vec![])
        } else {
            Vec::with_capacity(tasks.len())
        };

        for task in tasks {
            results.push(task.await??);
        }

        Ok(results)
    }

    pub async fn upload(
        upload_type: UploadType,
        name: Arc<String>,
        part: usize,
        auth_token: Arc<String>,
        data: Arc<Vec<u8>>,
        progress_listener: Option<ProgressListener>,
    ) -> Result<Cid, Error> {
        let api = Arc::new(format!("https://api.web3.storage/{}", upload_type));

        let upload_fn = || {
            let body = Body::wrap_stream(ProgressStream {
                name: name.clone(),
                part,
                data: data.clone(),
                cursor: 0,
                progress_listener: progress_listener.clone(),
            });

            Client::new()
                .post(api.clone().as_str())
                .header("X-NAME", name.clone().as_str())
                .header("accept", "application/json")
                .bearer_auth(auth_token.clone())
                .body(body)
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
        self.pause_to_complete_tasks();

        let upload_future = Uploader::upload(
            self.upload_type,
            self.name.clone(),
            self.tasks.len(),
            self.auth_token.clone(),
            Arc::new(buf.to_vec()),
            self.progress_listener.clone(),
        );
        let handler = tokio::spawn(upload_future);
        self.tasks.push(handler);

        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Deserialize)]
struct Response {
    cid: String,
}

#[derive(Clone)]
pub struct ProgressStream {
    name: Arc<String>,
    part: usize,
    data: Arc<Vec<u8>>,
    cursor: usize,
    progress_listener: Option<ProgressListener>,
}
impl futures::Stream for ProgressStream {
    type Item = io::Result<Vec<u8>>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let total_len = self.data.len();
        let remain_len = total_len - self.cursor;

        if remain_len == 0 {
            Poll::Ready(None)
        } else {
            let mut result = vec![0u8; cmp::min(remain_len, 1024 * 32)];
            let start_index = self.cursor;
            self.cursor += result.len();
            result.copy_from_slice(&self.data[start_index..self.cursor]);

            if let Some(pl) = self.progress_listener.as_ref() {
                if let Ok(mut f) = pl.lock() {
                    f(self.name.clone(), self.part, self.cursor, total_len);
                }
            }

            Poll::Ready(Some(Ok(result)))
        }
    }
}
