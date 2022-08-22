use super::*;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::rc::Rc;
use car_util::DirectoryItem;

pub struct Dir<W: io::Write> {
    curr_file: Rc<RefCell<u64>>,
    next_writer: W,
}

impl<W: io::Write> Dir<W> {
    pub fn new(next_writer: W) -> Self {
        Dir { curr_file: Rc::new(RefCell::new(0)), next_writer }
    }

    pub fn walk_write(&mut self, dir_items: &[DirectoryItem]) -> io::Result<()> {
        for item in dir_items {
            match item {
                DirectoryItem::File(_, path, id) => {
                    *self.curr_file.borrow_mut() = *id;
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
}
