use cid::Cid;
use futures::Stream;
use tokio::io::AsyncRead;

use super::{
    error::Error,
    header::CarHeader,
    util::{ld_read, read_node},
};

/// Reads CAR files that are in a BufReader
pub struct CarReader<R> {
    reader: R,
    header: CarHeader,
    buffer: Vec<u8>,
}

impl<R> CarReader<R>
where
    R: AsyncRead + Send + Unpin,
{
    /// Creates a new CarReader and parses the CarHeader
    pub async fn new(mut reader: R) -> Result<Self, Error> {
        let mut buffer = Vec::new();

        if !ld_read(&mut reader, &mut buffer).await? {
            return Err(Error::Parsing(
                "failed to parse uvarint for header".to_string(),
            ));
        }

        let header = CarHeader::decode(&buffer)?;

        Ok(CarReader {
            reader,
            header,
            buffer,
        })
    }

    /// Returns the header of this car file.
    pub fn header(&self) -> &CarHeader {
        &self.header
    }

    /// Returns the next IPLD Block in the buffer
    pub async fn next_block(&mut self) -> Result<Option<(Cid, Vec<u8>)>, Error> {
        read_node(&mut self.reader, &mut self.buffer).await
    }

    pub fn stream(self) -> impl Stream<Item = Result<(Cid, Vec<u8>), Error>> {
        futures::stream::try_unfold(self, |mut this| async move {
            let maybe_block = read_node(&mut this.reader, &mut this.buffer).await?;
            Ok(maybe_block.map(|b| (b, this)))
        })
    }
}
