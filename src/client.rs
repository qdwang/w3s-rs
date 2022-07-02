use super::*;
use std::io;
use thiserror::Error;
use tokio::task::JoinHandle;

pub trait ChunkUpload: io::Write {
    fn has_remain_buf(&self) -> bool;
    fn get_spawned_tasks(&self) -> Vec<JoinHandle<Result<String, Error>>>;
    fn spawn_new_upload(&mut self);
}
pub trait Encode<CU: ChunkUpload>: io::Write {
    fn done(self) -> io::Result<CU>;
}

pub struct Client {
    auth_token: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error")]
    IOError(#[from] io::Error),
    #[error("Tokio Task Join Error")]
    TaskJoinError(#[from] tokio::task::JoinError),
}

impl Client {
    pub async fn upload<CU: ChunkUpload>(
        &self,
        name: String,
        mut reader: impl io::Read,
        mut encoder: impl Encode<CU>,
    ) -> Result<Vec<String>, Error> {
        io::copy(&mut reader, &mut encoder)?;
        let mut cu = encoder.done()?;
        
        if cu.has_remain_buf() {
            cu.spawn_new_upload();
        }

        let tasks = cu.get_spawned_tasks();

        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks {
            let result = task.await??;
            results.push(result);
        }

        Ok(results)
    }
    pub async fn check_uploads(
        &self,
        before: Option<String>,
    ) -> Result<Vec<api::StorageItem>, Error> {
        unimplemented!()
    }
    pub async fn check_all_uploads(&self) -> Result<Vec<api::StorageItem>, Error> {
        unimplemented!()
    }
}
