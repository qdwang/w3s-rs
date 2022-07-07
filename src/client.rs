use super::*;
use std::io;
use thiserror::Error;

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
    pub async fn upload<W: io::Write>(
        &self,
        name: String,
        mut reader: impl io::Read,
        mut chain_writer: impl writer::ChainWrite<W>,
    ) -> Result<Vec<String>, Error> {
        unimplemented!()
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
