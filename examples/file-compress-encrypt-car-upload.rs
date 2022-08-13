use anyhow::Result;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use w3s::writer::crypto::Cipher;
use w3s::writer::{car, uploader, ChainWrite};

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
            "
        Please input file path and web3.storage auth token
        Example:
            cargo run --all-features --example file-compress-encrypt-car-upload the/path/to/my_file eyJhbG......MHlq0
        "
        ),
    }
}

async fn upload(path: &String, auth_token: &String) -> Result<()> {
    let mut file = File::open(path)?;
    let filename = get_file_name(path).unwrap();

    let uploader = uploader::Uploader::new(
        auth_token.clone(),
        filename.clone(),
        uploader::UploadType::Car,
        2,
        Some(Arc::new(Mutex::new(|name, part, pos, total| {
            println!("name: {name} part:{part} {pos}/{total}");
        }))),
    );
    let car = car::Car::new(filename, 1024 * 1024, uploader);

    let mut pwd = b"abcd1234".to_owned();
    let cipher = Cipher::new(&mut pwd, car)?;

    let mut compressor = zstd::stream::Encoder::new(cipher, 10)?;
    io::copy(&mut file, &mut compressor)?;
    compressor.flush()?;
    let mut cipher = compressor.finish()?;
    cipher.flush()?;

    let mut uploader = cipher.next().next();
    let results = uploader.finish_results().await?;
    println!("results: {:?}", results);

    Ok(())
}
