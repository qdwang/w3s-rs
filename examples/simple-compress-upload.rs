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
                "compressed_filename".to_owned(),
                uploader::UploadType::Upload,
                2,
                Some(Arc::new(Mutex::new(|name, part, pos, total| {
                    println!("name: {name} part:{part} {pos}/{total}");
                }))),
            );
            let splitter = splitter::PlainSplitter::new(uploader);

            // need feature `zstd`
            let mut compressor = zstd::stream::Encoder::new(splitter, 10)?;
            io::copy(&mut file, &mut compressor)?;

            let mut splitter = compressor.finish()?;
            splitter.flush()?; // this line is needed to upload the final part of the file

            let mut uploader = w3s::take_nth_writer!(splitter);
            let results = uploader.finish_results().await?;
            println!("results: {:?}", results);

            Ok(())
        }
        _ => panic!(
            "
        Please input file path and web3.storage auth token
        Example:
            cargo run --all-features --example simple-compress-upload the/path/to/my_file eyJhbG......MHlq0
        "
        ),
    }
}
