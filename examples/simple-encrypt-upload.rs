use anyhow::Result;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use w3s::writer::{splitter, uploader, crypto::Cipher};

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, path, auth_token] => upload(path, auth_token).await,
        _ => panic!(
            "
        Please input file path and web3.storage auth token
        Example:
            cargo run --all-features --example simple-encrypt-upload the/path/to/my_file eyJhbG......MHlq0
        "
        ),
    }
}

async fn upload(path: &String, auth_token: &String) -> Result<()> {
    let mut file = File::open(path)?;

    let uploader = uploader::Uploader::new(
        auth_token.clone(),
        "encrypted_filename".to_owned(),
        uploader::UploadType::Upload,
        2,
        Some(Arc::new(Mutex::new(|name, part, pos, total| {
            println!("name: {name} part:{part} {pos}/{total}");
        }))),
    );
    let splitter = splitter::PlainSplitter::new(uploader);
    
    let mut pwd = b"abcd1234".to_owned();
    // need feature `encryption`
    let mut cipher = Cipher::new(&mut pwd, splitter)?;

    io::copy(&mut file, &mut cipher)?;
    cipher.flush()?;

    let mut uploader = w3s::take_nth_writer!(cipher>);
    let results = uploader.finish_results().await?;
    println!("results: {:?}", results);

    Ok(())
}
