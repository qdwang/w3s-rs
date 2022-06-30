use super::*;
use thiserror::Error;
use std::io;

pub trait ChunkUpload : io::Write {
    fn spawn_new_upload(&mut self);
}
pub trait Encode<CU: ChunkUpload> {
    fn done(self) -> io::Result<CU>;
}

pub struct Client {
    auth_token: String,
}


#[derive(Error, Debug)]
pub enum Error {}

impl Client {
    pub async fn upload<CU: ChunkUpload>(&self, name: String, reader: impl io::Read, encoder: impl Encode<CU>) -> Result<Vec<String>, Error> {
        unimplemented!()
    }
    pub async fn check_uploads(&self, before: Option<String>) -> Result<Vec<api::StorageItem>, Error> {
        unimplemented!()
    }
    pub async fn check_all_uploads(&self) -> Result<Vec<api::StorageItem>, Error> {
        unimplemented!()
    }
}

