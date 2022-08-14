use cid::Cid;

use super::writer::*;
use std::io::{self, Write};


pub async fn upload(
    auth_token: impl AsRef<str>,
    name: impl AsRef<str>,
    max_upload_concurrent: usize,
    progress_listener: Option<uploader::ProgressListener>,
    reader: &mut impl io::Read,
) -> Result<Vec<Cid>, uploader::Error> {
    let uploader = uploader::Uploader::new(
        auth_token.as_ref().to_owned(),
        name.as_ref().to_owned(),
        uploader::UploadType::Upload,
        max_upload_concurrent,
        progress_listener,
    );

    let mut splitter = splitter::PlainSplitter::new(uploader);

    io::copy(reader, &mut splitter)?;
    splitter.flush()?;

    let mut uploader = splitter.next();
    let results = uploader.finish_results().await?;

    Ok(results)
}

