use super::*;
use car_util::DirectoryItem;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct Dir<W: io::Write> {
    curr_name: Arc<Mutex<String>>,
    curr_file_id: Rc<RefCell<u64>>,
    next_writer: W,
}

impl<W: io::Write> Dir<W> {
    pub fn new(
        curr_name: Arc<Mutex<String>>,
        curr_file_id: Rc<RefCell<u64>>,
        next_writer: W,
    ) -> Self {
        Dir {
            curr_name,
            curr_file_id,
            next_writer,
        }
    }

    pub fn walk_write(&mut self, dir_items: &[DirectoryItem]) -> io::Result<()> {
        for item in dir_items {
            match item {
                DirectoryItem::File(name, path, id) => {
                    if let Ok(mut mutex_name) = self.curr_name.lock() {
                        *mutex_name = name.to_owned();
                    }
                    
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

    pub fn next(self) -> W {
        self.next_writer
    }
}
