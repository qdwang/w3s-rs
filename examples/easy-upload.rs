use anyhow::Result;
use std::env;
use std::fs::File;
use std::sync::{Arc, Mutex};
use w3s::helper;

fn get_file_name(path: &String) -> Option<String> {
    let path = std::path::Path::new(path);
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|x| Some(x.to_owned()))
}

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
    let mut file = File::open(path)?;
    let filename = get_file_name(path).unwrap();

    let pwd = b"abcd1234".to_owned();
    let results = helper::upload(
        &mut file,
        auth_token,
        filename,
        2,
        Some(Arc::new(Mutex::new(|name, part, pos, total| {
            println!("name: {name} part:{part} {pos}/{total}");
        }))),
        Some(None),
        Some(pwd),
        Some(None),
    ).await?;
    
    println!("results: {:?}", results);

    Ok(())
}
