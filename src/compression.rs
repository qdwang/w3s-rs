use super::*;
use std::io;
use zstd::Encoder;
use tokio::task::JoinHandle;

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
    fn get_spawned_tasks(&self) -> Vec<JoinHandle<Result<String, client::Error>>> {
        unimplemented!()
    }
    fn has_remain_buf(&self) -> bool {
        unimplemented!()
    }
    fn spawn_new_upload(&mut self) {
        unimplemented!()
    }
}

impl<'a> client::Encode<ZstdUpload> for Encoder<'a, ZstdUpload> {
    fn done(self) -> io::Result<ZstdUpload> {
        self.finish()
    }
}
