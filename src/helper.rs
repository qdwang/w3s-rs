use cid::Cid;
use thiserror::Error;

use super::writer::*;
use std::io::{self, Write};

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("Upload error")]
    UploadError(#[from] uploader::Error),
    #[error("Cipher error")]
    CipherError(#[from] crypto::Error),
}

fn gen_uploader(
    auth_token: impl AsRef<str>,
    name: impl AsRef<str>,
    max_upload_concurrent: usize,
    progress_listener: Option<uploader::ProgressListener>,
    with_car: Option<Option<usize>>,
) -> Box<dyn ChainWrite<uploader::Uploader>> {
    let uploader = uploader::Uploader::new(
        auth_token.as_ref().to_owned(),
        name.as_ref().to_owned(),
        if with_car.is_some() {
            uploader::UploadType::Car
        } else {
            uploader::UploadType::Upload
        },
        max_upload_concurrent,
        progress_listener,
    );

    if let Some(custom_block_size) = with_car {
        Box::new(car::Car::new(
            name.as_ref().to_owned(),
            custom_block_size.unwrap_or(1024 * 1024),
            uploader,
        ))
    } else {
        Box::new(splitter::PlainSplitter::new(uploader))
    }
}

pub async fn upload(
    reader: &mut impl io::Read,
    auth_token: impl AsRef<str>,
    name: impl AsRef<str>,
    max_upload_concurrent: usize,
    progress_listener: Option<uploader::ProgressListener>,
    with_car: Option<Option<usize>>,
    with_encryption: Option<impl AsMut<[u8]>>,
    with_compression: Option<Option<i32>>,
) -> Result<Vec<Cid>, Error> {
    let mut writer = gen_uploader(
        auth_token,
        name,
        max_upload_concurrent,
        progress_listener,
        with_car,
    );

    let results = match (with_compression, with_encryption) {
        (Some(level), Some(mut password)) => {
            let cipher = crypto::Cipher::new(password.as_mut(), writer)?;
            let mut compressor = zstd::stream::Encoder::new(cipher, level.unwrap_or(10))?;
            io::copy(reader, &mut compressor)?;
            let mut cipher = compressor.finish()?;
            cipher.flush()?;
            cipher.next_writer().next_writer().finish_results().await?
        }
        (Some(level), None) => {
            let mut compressor = zstd::stream::Encoder::new(writer, level.unwrap_or(10))?;
            io::copy(reader, &mut compressor)?;
            let mut writer = compressor.finish()?;
            writer.flush()?;
            writer.next_writer().finish_results().await?
        }
        (None, Some(mut password)) => {
            let mut cipher = crypto::Cipher::new(password.as_mut(), writer)?;
            io::copy(reader, &mut cipher)?;
            cipher.flush()?;
            cipher.next_writer().next_writer().finish_results().await?
        }
        _ => {
            io::copy(reader, &mut writer)?;
            writer.next_writer().flush()?;
            writer.next_writer().finish_results().await?
        }
    };

    Ok(results)
}
