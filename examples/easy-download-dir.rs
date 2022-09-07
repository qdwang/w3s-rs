use anyhow::Result;
use std::env;
use std::sync::{Arc, Mutex};
use w3s::helper;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, url, path] => download(url, path).await,
        _ => panic!("\n\nPlease input [url_to_the_ipfs_dir] and the [path_to_save_dir]\n\n"),
    }
}

async fn download(url: &String, path: &String) -> Result<()> {
    helper::download_dir(
        url,
        path,
        Some(|url, status| println!("checked: {url} -> {status}")),
        Some(Arc::new(Mutex::new(|name, _, pos, total| {
            println!("name: {name} {pos}/{total}");
        }))),
        None,
        false,
    )
    .await?;

    println!("dir downloaded to path: {path}");

    Ok(())
}
