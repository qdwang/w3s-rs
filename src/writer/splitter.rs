use super::*;
use std::io;
use std::mem;

const MAX_CHUNK_SIZE: usize = 104752742; // 99.9mb

pub struct PlainSplitter<W: io::Write> {
    chunk: Vec<u8>,
    next_writer: W,
}

impl<W: io::Write> PlainSplitter<W> {
    pub fn new(next_writer: W) -> Self {
        PlainSplitter {
            chunk: vec![],
            next_writer,
        }
    }
}

impl<W: io::Write> io::Write for PlainSplitter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.chunk.extend(buf);
        if self.chunk.len() > MAX_CHUNK_SIZE {
            let written_len = {
                let chunk = mem::replace(&mut self.chunk, vec![]);
                self.write2next(&chunk)?
            };

            // the written length result won't propagate
            // abnormal written len can cause the flush function from the next writer to do some additional check, async await or pause needs.
            // so if the `write` implementation of a Writer can return 0, it means it's `flush` function is used for additional usage.
            if written_len == 0 {
                self.next_writer().flush()?;
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        {
            let chunk = mem::replace(&mut self.chunk, vec![]);
            self.write2next(&chunk)?;
        }
        self.next_writer().flush()?;
        Ok(())
    }
}

impl<W: io::Write> ChainWrite<W> for PlainSplitter<W> {
    fn next(self) -> W {
        self.next_writer
    }
    fn next_writer(&mut self) -> &mut W {
        &mut self.next_writer
    }
}
