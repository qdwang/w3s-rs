use super::*;
use zstd::Encoder;
use std::io;

pub struct ZstdUpload;
impl io::Write for ZstdUpload {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(3)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl client::ChunkUpload for ZstdUpload {
    fn spawn_new_upload(&mut self) {
        
    }
}

impl<'a> client::Encode<ZstdUpload> for Encoder<'a, ZstdUpload> {
    fn done(self) -> io::Result<ZstdUpload> {
        self.finish()
    }
}