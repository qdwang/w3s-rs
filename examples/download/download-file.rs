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
            "\n\nPlease input [ipfs_file_url] and the [path_to_save_the_file]\n\n"
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
        .download(Arc::new(path.clone()), url.as_str(), start_offset)
        .await?;

    println!("file downloaded to path:{path}");
    Ok(())
}
