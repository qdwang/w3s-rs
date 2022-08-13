use anyhow::Result;
use std::fs::{OpenOptions};
use std::sync::{Arc, Mutex};
use std::{env};
use w3s::writer::crypto::Cipher;
use w3s::writer::decompressor;
use w3s::writer::downloader;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, url, path] => download(url, path).await,
        _ => panic!(
            "
        Please input encrypted ipfs file url and the path to save the file
        Example:
            cargo run --all-features --example download-decrypt-decompress-file url_to_the_encrypted_ipfs_file path_to_save_file
        "
        ),
    }
}

async fn download(url: &String, path: &String) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open(path)?;

    let decompressor = decompressor::Decompressor::new(file)?;
    let cipher = Cipher::new_decryption(b"abcd1234".to_vec(), decompressor)?;

    let mut downloader = downloader::Downloader::new(
        Some(Arc::new(Mutex::new(|name, _, pos, total| {
            println!("name: {name} {pos}/{total}");
        }))),
        cipher,
    );
    downloader
        .download(Arc::new("file1".to_owned()), url.as_str(), None)
        .await?;

    println!("file1 downloaded to path:{path}");
    Ok(())
}
