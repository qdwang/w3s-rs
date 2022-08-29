use anyhow::Result;
use std::env;
use std::fs::OpenOptions;
use std::sync::{Arc, Mutex};
use w3s::writer::cipher::Cipher;
use w3s::writer::decompressor;
use w3s::writer::downloader;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, url, path] => download(url, path).await,
        _ => panic!(
            "\n\nPlease input [url_to_the_encrypted_ipfs_file] and the [path_to_save_file]\n\n"
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
        .download(path.clone(), url.as_str(), None)
        .await?;

    println!("file downloaded to path:{path}");
    Ok(())
}
