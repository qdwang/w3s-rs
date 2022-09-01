use cid::Cid;
use core::task::Poll;
use futures::TryFutureExt;
use reqwest::{Body, Client};
use serde::Deserialize;
use std::{
    cmp, fmt,
    io, mem,
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
    #[error("Serde JSON error. Response: {1}")]
    SerdeJSONError(#[source] serde_json::Error, String),
    #[error("Cid parsing error")]
    CidError(#[from] cid::Error),
    #[error("IO error")]
    IoError(#[from] io::Error),
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

pub type ProgressListener =
    Arc<Mutex<dyn FnMut(Arc<String>, usize, usize, usize) + Send + Sync + 'static>>;
pub struct Uploader {
    upload_type: UploadType,
    auth_token: Arc<String>,
    w3s_name: Arc<String>,
    max_concurrent: usize,
    tasks: Vec<JoinHandle<Result<Cid, Error>>>,
    results: Vec<Cid>,
    progress_listener: Option<ProgressListener>,
}

impl Uploader {
    pub fn new(
        auth_token: String,
        w3s_name: String,
        upload_type: UploadType,
        max_concurrent: usize,
        progress_listener: Option<ProgressListener>,
    ) -> Self {
        Uploader {
            upload_type,
            auth_token: Arc::new(auth_token),
            w3s_name: Arc::new(w3s_name),
            max_concurrent,
            tasks: vec![],
            results: vec![],
            progress_listener,
        }
    }

    pub fn pause_to_complete_tasks(&mut self) -> Result<(), Error> {
        if self.tasks.len() == self.max_concurrent {
            tokio::task::block_in_place(|| -> Result<(), Error> {
                let cid = Handle::current().block_on(self.finish_any_result())?;
                self.results.push(cid);
                Ok(())
            })?;
        }

        Ok(())
    }

    pub async fn finish_results(&mut self) -> Result<Vec<Cid>, Error> {
        let tasks = mem::take(&mut self.tasks);

        let mut results = mem::take(&mut self.results);

        for task in tasks {
            results.push(task.await??);
        }

        Ok(results)
    }

    pub async fn finish_any_result(&mut self) -> Result<Cid, Error> {
        let tasks = mem::take(&mut self.tasks);

        let (result, remnant) = futures::future::select_ok(tasks).await?;

        self.tasks = remnant;

        result
    }

    pub async fn upload(
        upload_type: UploadType,
        w3s_name: Arc<String>,
        part: usize,
        auth_token: Arc<String>,
        data: Arc<Vec<u8>>,
        progress_listener: Option<ProgressListener>,
    ) -> Result<Cid, Error> {
        let api = Arc::new(format!("https://api.web3.storage/{}", upload_type));

        let upload_fn = || {
            let body = Body::wrap_stream(ProgressStream {
                name: w3s_name.clone(),
                part,
                data: data.clone(),
                cursor: 0,
                progress_listener: progress_listener.clone(),
            });

            Client::new()
                .post(api.clone().as_str())
                .header("X-NAME", w3s_name.clone().as_str())
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
        let upload_future = Uploader::upload(
            self.upload_type,
            self.w3s_name.clone(),
            self.tasks.len() + self.results.len(),
            self.auth_token.clone(),
            Arc::new(buf.to_vec()),
            self.progress_listener.clone(),
        );
        let handler = tokio::spawn(upload_future);
        self.tasks.push(handler);

        if self.tasks.len() == self.max_concurrent {
            // abnormal written len can tell the parent Writer to call `flush` after this `write` function.
            // you shouldn't call `self.flush()` directly here because it can't drop the outside Vec to release memory
            // when pause the thread to await async uploading jobs.
            Ok(0)
        } else {
            Ok(buf.len())
        }
    }

    // this `flush` function is to complete concurrent uploading connections by blocking current thread.
    fn flush(&mut self) -> io::Result<()> {
        self.pause_to_complete_tasks()
            .map_err(|e| io::Error::new(io::ErrorKind::Interrupted, e))?;
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
