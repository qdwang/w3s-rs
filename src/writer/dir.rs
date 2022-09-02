//! Handles recursive walks in a directory and writes file bytes to the next writer
//! 
use super::*;
use car_util::DirectoryItem;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::rc::Rc;

pub struct Dir<W: io::Write> {
    curr_file_id: Rc<RefCell<u64>>,
    next_writer: W,
}

impl<W: io::Write> Dir<W> {
    pub fn new(curr_file_id: Rc<RefCell<u64>>, next_writer: W) -> Self {
        Dir {
            curr_file_id,
            next_writer,
        }
    }

    pub fn walk_write(&mut self, dir_items: &[DirectoryItem]) -> io::Result<()> {
        for item in dir_items {
            match item {
                DirectoryItem::File(_, path, id) => {
                    *self.curr_file_id.borrow_mut() = *id;
                    let mut file = File::open(path)?;
                    io::copy(&mut file, &mut self.next_writer)?;
                    self.next_writer.flush()?;
                }
                DirectoryItem::Directory(_, sub_dir_items) => {
                    self.walk_write(sub_dir_items)?;
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "zstd")]
    pub fn walk_write_with_compression(
        &mut self,
        dir_items: &[DirectoryItem],
        level: Option<i32>,
    ) -> io::Result<()> {
        for item in dir_items {
            match item {
                DirectoryItem::File(_, path, id) => {
                    *self.curr_file_id.borrow_mut() = *id;
                    let mut file = File::open(path)?;

                    let mut compressor =
                        zstd::stream::Encoder::new(&mut self.next_writer, level.unwrap_or(10))?;
                    io::copy(&mut file, &mut compressor)?;

                    compressor.finish()?;
                    self.next_writer.flush()?;
                }
                DirectoryItem::Directory(_, sub_dir_items) => {
                    self.walk_write_with_compression(sub_dir_items, level)?;
                }
            }
        }

        Ok(())
    }

    pub fn next(self) -> W {
        self.next_writer
    }
}
