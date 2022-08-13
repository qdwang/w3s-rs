use super::*;
use std::io::{self};
use zstd::stream::write::Decoder;

pub struct Decompressor<'a, W: io::Write> {
    cache: Vec<u8>,
    next_writer: Decoder<'a, W>,
}

impl<'a, W: io::Write> Decompressor<'a, W> {
    pub fn new(next_writer: W) -> io::Result<Self> {
        Ok(Decompressor {
            cache: vec![],
            next_writer: Decoder::new(next_writer)?,
        })
    }
}
impl<'a, W: io::Write> io::Write for Decompressor<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.cache.extend(buf);

        let len = self.next_writer.write(&self.cache)?;
        self.cache.drain(0..len);

        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.next_writer.flush()
    }
}

impl<'a, W: io::Write> ChainWrite<W> for Decompressor<'a, W> {
    fn next(self) -> W {
        self.next_writer.into_inner()
    }
    fn next_writer(&mut self) -> &mut W {
        self.next_writer.get_mut()
    }
}
