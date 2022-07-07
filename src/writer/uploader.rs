use super::*;
use std::io;

pub struct Uploader {
    pub tasks: Vec<usize>
}

impl Uploader {
    pub fn new() -> Self {
        Uploader { tasks: vec![] }
    }
}

impl io::Write for Uploader {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tasks.push(buf.len());
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}