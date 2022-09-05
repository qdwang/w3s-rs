use anyhow::Result;
use std::env;
use std::sync::{Arc, Mutex};
use w3s::helper;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, path, auth_token] => upload(path, auth_token).await,
        _ => panic!(
            "\n\nPlease input [the_path_to_the_file] and [web3.storage_auth_token(eyJhbG......MHlq0)]\n\n"
        ),
    }
}

async fn upload(path: &String, auth_token: &String) -> Result<()> {
    let results = helper::upload(
        path,
        auth_token,
        2,
        Some(Arc::new(Mutex::new(|name, part, pos, total| {
            println!("name: {name} part:{part} {pos}/{total}");
        }))),
        Some(None),
        Some(b"abcd1234".to_vec()),
        Some(None),
    )
    .await?;

    println!("results: {:?}", results);

    Ok(())
}
