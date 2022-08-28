use cid::Cid;
use thiserror::Error;

use crate::writer::car_util::DirectoryItem;

use super::writer::*;
use std::cell::RefCell;
use std::io::{self, Write};

use std::rc::Rc;
use std::sync::Arc;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("Upload error")]
    UploadError(#[from] uploader::Error),

    #[cfg(feature = "encryption")]
    #[error("Cipher error")]
    CipherError(#[from] cipher::Error),

    #[error("Download error")]
    DownloadError(#[from] downloader::Error),
    #[error("The feature:\"encryption\" is required.")]
    FeatureNoCipher,
    #[error("The feature:\"zstd\" is required.")]
    FeatureNoZstd,
    #[error("The features:\"encryption zstd\" are required.")]
    FeatureNoCipherAndZstd,
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

    let dir_item = car::SingleFileToDirectoryItem(name.as_ref(), None);

    if let Some(custom_block_size) = with_car {
        Box::new(car::Car::new(
            1,
            Rc::new(vec![dir_item]),
            None,
            custom_block_size,
            uploader,
        ))
    } else {
        Box::new(splitter::PlainSplitter::new(uploader))
    }
}

#[cfg(all(feature = "zstd", feature = "encryption"))]
async fn compress_then_encrypt<'a>(
    reader: &'a mut impl io::Read,
    writer: Box<dyn ChainWrite<uploader::Uploader>>,
    level: Option<i32>,
    password: &'a mut [u8],
) -> Result<Vec<Cid>, Error> {
    let cipher = cipher::Cipher::new(password, writer)?;
    let mut compressor = zstd::stream::Encoder::new(cipher, level.unwrap_or(10))?;
    io::copy(reader, &mut compressor)?;
    let mut cipher = compressor.finish()?;
    cipher.flush()?;
    let ret = cipher.next_mut().next_mut().finish_results().await?;
    Ok(ret)
}
#[cfg(not(all(feature = "zstd", feature = "encryption")))]
async fn compress_then_encrypt<'a>(
    _: &'a mut impl io::Read,
    _: Box<dyn ChainWrite<uploader::Uploader>>,
    _: Option<i32>,
    _: &'a mut [u8],
) -> Result<Vec<Cid>, Error> {
    Err(Error::FeatureNoCipherAndZstd)
}

#[cfg(feature = "zstd")]
async fn compress(
    reader: &mut impl io::Read,
    writer: Box<dyn ChainWrite<uploader::Uploader>>,
    level: Option<i32>,
) -> Result<Vec<Cid>, Error> {
    let mut compressor = zstd::stream::Encoder::new(writer, level.unwrap_or(10))?;
    io::copy(reader, &mut compressor)?;
    let mut writer = compressor.finish()?;
    writer.flush()?;
    let ret = writer.next_mut().finish_results().await?;
    Ok(ret)
}
#[cfg(not(feature = "zstd"))]
async fn compress(
    _: &mut impl io::Read,
    _: Box<dyn ChainWrite<uploader::Uploader>>,
    _: Option<i32>,
) -> Result<Vec<Cid>, Error> {
    Err(Error::FeatureNoZstd)
}

#[cfg(feature = "encryption")]
async fn encrypt<'a>(
    reader: &'a mut impl io::Read,
    writer: Box<dyn ChainWrite<uploader::Uploader>>,
    password: &'a mut [u8],
) -> Result<Vec<Cid>, Error> {
    let mut cipher = cipher::Cipher::new(password, writer)?;
    io::copy(reader, &mut cipher)?;
    cipher.flush()?;
    let ret = cipher.next_mut().next_mut().finish_results().await?;
    Ok(ret)
}
#[cfg(not(feature = "encryption"))]
async fn encrypt<'a>(
    _: &'a mut impl io::Read,
    _: Box<dyn ChainWrite<uploader::Uploader>>,
    _: &'a mut [u8],
) -> Result<Vec<Cid>, Error> {
    Err(Error::FeatureNoCipher)
}

pub async fn upload_dir(
    dir_path: &str,
    auth_token: String,
    max_upload_concurrent: usize,
    progress_listener: Option<uploader::ProgressListener>,
    with_encryption: Option<&mut [u8]>,
    with_compression: Option<Option<i32>>,
) -> Result<Vec<Cid>, Error> {
    let uploader = uploader::Uploader::new(
        auth_token,
        dir_path.to_owned(),
        uploader::UploadType::Car,
        max_upload_concurrent,
        progress_listener,
    );

    let (dir_items, count) = DirectoryItem::from_path(dir_path, None)?;
    let dir_items_rc = Rc::new(dir_items);

    let curr_file_id = Rc::new(RefCell::new(0));

    let car = car::Car::new(
        count as usize,
        dir_items_rc.clone(),
        Some(curr_file_id.clone()),
        None,
        uploader,
    );
    let mut dir = dir::Dir::new(curr_file_id, car);

    dir.walk_write(&dir_items_rc)?;

    let results = dir.next().next().finish_results().await?;

    Ok(results)
}

pub async fn upload(
    reader: &mut impl io::Read,
    auth_token: impl AsRef<str>,
    name: impl AsRef<str>,
    max_upload_concurrent: usize,
    progress_listener: Option<uploader::ProgressListener>,
    with_car: Option<Option<usize>>,
    with_encryption: Option<&mut [u8]>,
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
        (Some(level), Some(password)) => {
            compress_then_encrypt(reader, writer, level, password).await?
        }
        (Some(level), None) => compress(reader, writer, level).await?,
        (None, Some(password)) => encrypt(reader, writer, password).await?,
        _ => {
            io::copy(reader, &mut writer)?;
            writer.next_mut().flush()?;
            writer.next_mut().finish_results().await?
        }
    };

    Ok(results)
}

#[cfg(all(feature = "zstd", feature = "encryption"))]
fn decrypt_then_decompress<'a>(
    writer: impl io::Write,
    password: Vec<u8>,
) -> Result<cipher::Cipher<decompressor::Decompressor<'a, impl io::Write>>, Error> {
    let decompressor = decompressor::Decompressor::new(writer)?;
    let cipher = cipher::Cipher::new_decryption(password, decompressor)?;
    Ok(cipher)
}
#[cfg(not(all(feature = "zstd", feature = "encryption")))]
fn decrypt_then_decompress<W: io::Write>(_: W, _: Vec<u8>) -> Result<W, Error> {
    Err(Error::FeatureNoCipherAndZstd)
}
#[cfg(feature = "encryption")]
fn decrypt(
    writer: impl io::Write,
    password: Vec<u8>,
) -> Result<cipher::Cipher<impl io::Write>, Error> {
    let cipher = cipher::Cipher::new_decryption(password, writer)?;
    Ok(cipher)
}
#[cfg(not(feature = "encryption"))]
fn decrypt<W: io::Write>(_: W, _: Vec<u8>) -> Result<W, Error> {
    Err(Error::FeatureNoCipher)
}
#[cfg(feature = "zstd")]
fn decompress<'a>(
    writer: impl io::Write,
) -> Result<decompressor::Decompressor<'a, impl io::Write>, Error> {
    let decompressor = decompressor::Decompressor::new(writer)?;
    Ok(decompressor)
}
#[cfg(not(feature = "zstd"))]
fn decompress<W: io::Write>(_: W) -> Result<W, Error> {
    Err(Error::FeatureNoZstd)
}

pub async fn download(
    url: impl AsRef<str>,
    name: impl AsRef<str>,
    writer: impl io::Write,
    progress_listener: Option<uploader::ProgressListener>,
    start_offset: Option<u64>,
    with_decryption: Option<Vec<u8>>,
    with_decompression: bool,
) -> Result<(), Error> {
    macro_rules! gen_downloader {
        ($writer:expr) => {{
            let mut downloader = downloader::Downloader::new(progress_listener, $writer);
            downloader
                .download(
                    Arc::new(name.as_ref().to_owned()),
                    url.as_ref(),
                    start_offset,
                )
                .await?;
        }};
    }

    match (with_decompression, with_decryption) {
        (true, Some(password)) => {
            let cipher = decrypt_then_decompress(writer, password)?;
            gen_downloader!(cipher);
        }
        (false, Some(password)) => {
            let cipher = decrypt(writer, password)?;
            gen_downloader!(cipher);
        }
        (true, None) => {
            let decompressor = decompress(writer)?;
            gen_downloader!(decompressor);
        }
        _ => {
            gen_downloader!(writer);
        }
    };

    Ok(())
}
