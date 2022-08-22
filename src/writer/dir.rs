use std::io;
use super::*;


pub struct Dir<W : io::Write> {
    next_writer: W
}

impl<W : io::Write> io::Write for Dir<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<W : io::Write> ChainWrite<W> for Dir<W> {
    fn next(self) -> W {
        self.next_writer
    }
    fn next_writer(&mut self) -> &mut W {
        &mut self.next_writer
    }
}