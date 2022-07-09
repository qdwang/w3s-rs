use anyhow::Result;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use w3s::writer::{splitter, uploader};

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, path, auth_token] => {
            let mut file = File::open(path)?;

            let uploader = uploader::Uploader::new(
                auth_token.clone(),
                "filename".to_owned(),
                uploader::UploadType::Upload,
                Some(Arc::new(Mutex::new(|name, part, pos, total| {
                    println!("name: {name} part:{part} {pos}/{total}");
                }))),
            );
            let mut splitter = splitter::PlainSplitter::new(uploader);

            io::copy(&mut file, &mut splitter)?;
            splitter.flush()?;

            let uploader = w3s::take_nth_writer!(splitter);
            let results = uploader.results().await?;
            println!("{:?}", results);

            Ok(())
        }
        _ => panic!(
            "
        Please input file path and web3.storage auth token
        Example:
            cargo run --example simple-upload the/path/to/my_file eyJhbG......MHlq0
        "
        ),
    }
}
