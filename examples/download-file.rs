use anyhow::Result;
use std::env;
use std::fs::OpenOptions;
use std::sync::{Arc, Mutex};
use w3s::writer::downloader;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, url, path] => download(url, path).await,
        _ => panic!(
            "
        Please input ipfs file url and the path to save the file
        Example:
            cargo run --example download-file url_to_the_ipfs_file path_to_save_file
        "
        ),
    }
}

async fn download(url: &String, path: &String) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(path)?;

    let start_offset = if let Ok(meta) = file.metadata() {
        Some(meta.len())
    } else {
        None
    };

    let mut downloader = downloader::Downloader::new(
        Some(Arc::new(Mutex::new(|name, _, pos, total| {
            println!("name: {name} {pos}/{total}");
        }))),
        file,
    );
    downloader
        .download(Arc::new("file1".to_owned()), url.as_str(), start_offset)
        .await?;

    println!("file1 downloaded to path:{path}");
    Ok(())
}
