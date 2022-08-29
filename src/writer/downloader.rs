use super::{uploader::ProgressListener, ChainWrite};
use reqwest::Client;
use thiserror::Error;

use std::{io, sync::Arc};

#[derive(Error, Debug)]
pub enum Error {
    #[error("The server response does not contain content-length for the body. url: {0}")]
    NoContentLength(String),
    #[error("The reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("The IO error: {0:?}")]
    IOError(#[from] io::Error),
}

pub struct Downloader<W: io::Write> {
    progress_listener: Option<ProgressListener>,
    next_writer: W,
}

impl<W: io::Write> Downloader<W> {
    pub fn new(progress_listener: Option<ProgressListener>, next_writer: W) -> Self {
        Downloader {
            progress_listener,
            next_writer,
        }
    }
}

pub async fn fetch_mac(url: &str) -> Result<Vec<u8>, Error> {
    let size = Client::new()
        .head(url)
        .send()
        .await?
        .headers()
        .get("content-length")
        .map(|x| x.to_str().unwrap_or("").parse::<u64>().unwrap_or(0))
        .ok_or_else(|| Error::NoContentLength(url.to_owned()))?;

    let resp = Client::new()
        .get(url)
        .header("Range", format!("bytes={}-{}", size - 16, size))
        .send()
        .await?
        .bytes()
        .await?;

    Ok(resp.to_vec())
}

impl<W: io::Write> Downloader<W> {
    pub async fn download(
        &mut self,
        name: String,
        url: &str,
        start_offset: Option<u64>,
    ) -> Result<(), Error> {
        let mut req_builder = Client::new().get(url);
        let begin_offset = if let Some(offset) = start_offset {
            req_builder = req_builder.header("Range", format!("bytes={}-", offset));
            offset as usize
        } else {
            0
        };
        let mut resp = req_builder.send().await?;

        let total_len = if let Some(content_range) = resp.headers().get("Content-Range") {
            if let Ok(content_range_str) = content_range.to_str() {
                content_range_str
                    .split('/')
                    .into_iter()
                    .last()
                    .map(|x| x.parse::<u64>().ok())
                    .flatten()
            } else {
                resp.content_length()
            }
        } else {
            resp.content_length()
        }
        .map(|v| v as usize)
        .unwrap_or(0);
        // .ok_or_else(|| Error::NoContentLength(url.to_owned()))?;

        if total_len == 0 || begin_offset != total_len {
            let mut written_len = begin_offset;
            while let Ok(Some(chunk)) = resp.chunk().await {
                self.next_writer.write(chunk.as_ref())?;
                written_len += chunk.len();

                if let Some(pl) = self.progress_listener.as_ref() {
                    if let Ok(mut f) = pl.lock() {
                        f(name.clone(), 0, written_len, total_len);
                    }
                }
            }
            self.next_writer.flush()?;
        }

        Ok(())
    }
}

impl<W :io::Write> io::Write for Downloader<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<W :io::Write> ChainWrite<W> for Downloader<W> {
    fn next(self) -> W {
        self.next_writer
    }
    fn next_mut(&mut self) -> &mut W {
        &mut self.next_writer
    }
}