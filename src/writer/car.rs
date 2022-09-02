//! Handles CAR file generation from bytes
//! 
use super::super::iroh_car;
use super::*;
use car_util::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, io, mem};

use cid::Cid;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("Car file writing error: {0:?}")]
    CarWriteError(#[from] iroh_car::Error),
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::Interrupted, e)
    }
}

pub struct Car<W: io::Write> {
    files_count: usize,
    remote_file_id: Rc<RefCell<u64>>,
    id_map: HashMap<u64, Vec<UnixFsStruct>>,
    dir_items: Rc<Vec<DirectoryItem>>,
    buf: Vec<u8>,
    blocks: Vec<(Cid, Vec<u8>)>,
    block_size: usize,
    next_writer: W,
}

pub fn single_file_to_directory_item(name: &str, path: Option<&str>) -> DirectoryItem {
    DirectoryItem::File(name.to_owned(), path.unwrap_or(name).to_owned(), 0)
}

impl<W: io::Write> Car<W> {
    pub fn new(
        files_count: usize,
        dir_items: Rc<Vec<DirectoryItem>>,
        remote_file_id: Option<Rc<RefCell<u64>>>,
        custom_block_size: Option<usize>,
        next_writer: W,
    ) -> Car<W> {
        let block_size = custom_block_size.unwrap_or(256 * 1024);
        let remote_file_id = remote_file_id.unwrap_or_else(|| Rc::new(RefCell::new(0)));

        Car {
            files_count,
            dir_items,
            remote_file_id,
            id_map: HashMap::new(),
            buf: Vec::with_capacity(block_size + block_size / 10),
            blocks: vec![],
            block_size,
            next_writer,
        }
    }

    fn gen_car_from_buf(&mut self, buf: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let remote_id = *self.remote_file_id.borrow();
        let mut blocks = gen_blocks(buf, self.block_size);

        self.blocks
            .extend(blocks.iter_mut().map(|x| x.rip_data_with_cid()));

        // insert blocks into id_map
        if let Some(struct_lst) = self.id_map.get_mut(&remote_id) {
            struct_lst.extend(blocks);
        } else {
            self.id_map.insert(remote_id, blocks);
        }

        if self.blocks.len() >= car_util::MAX_CAR_SIZE / self.block_size {
            let mut blocks = mem::take(&mut self.blocks);
            let remain_blocks = blocks.split_off(car_util::MAX_CAR_SIZE / self.block_size);
            let car = gen_car_by_data(blocks, None)?;
            self.blocks = remain_blocks;
            Ok(Some(car))
        } else {
            Ok(None)
        }
    }
    fn buf_extend(&mut self, buf: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.buf.extend(buf);

        let prepared_buf = if self.buf.len() >= MAX_CAR_SIZE {
            Some(mem::take(&mut self.buf))
        } else {
            None
        };

        if let Some(buf) = prepared_buf {
            self.gen_car_from_buf(buf)
        } else {
            Ok(None)
        }
    }
}

impl<W: io::Write> io::Write for Car<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = self.buf_extend(buf)?;

        // this scope is for memory release for car(Vec<u8>)
        let need_flush = if let Some(car) = result {
            self.next_mut().write(&car)? == 0
        } else {
            false
        };

        if need_flush {
            self.next_mut().flush()?;
        }

        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if !self.buf.is_empty() {
            let remain_buf = mem::take(&mut self.buf);
            let car_data = self.gen_car_from_buf(remain_buf)?;
            if let Some(car) = car_data {
                if self.next_mut().write(&car)? == 0 {
                    self.next_mut().flush()?;
                }
            }
        }

        // final flush
        if self.files_count == self.id_map.len() {
            let mut blocks = vec![];
            let root_blocks: Vec<_> = self
                .dir_items
                .iter()
                .map(|item| item.to_unixfs_struct(&self.id_map, &mut blocks))
                .collect();

            let root = gen_dir(None, &root_blocks);

            self.blocks
                .extend(blocks.iter_mut().map(|x| x.rip_data_with_cid()));

            let car = gen_car_by_data(mem::take(&mut self.blocks), Some(root))
                .map_err(|e| io::Error::new(io::ErrorKind::Interrupted, e))?;

            let _ = self.next_mut().write(&car)?;
            self.next_mut().flush()?;
        }

        Ok(())
    }
}

impl<W: io::Write> ChainWrite<W> for Car<W> {
    fn next(self) -> W {
        self.next_writer
    }

    fn next_mut(&mut self) -> &mut W {
        &mut self.next_writer
    }
}
