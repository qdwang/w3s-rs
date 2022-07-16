use super::super::iroh_car;
use super::*;
use std::{borrow::Cow, io, mem};

use cid::Cid;
use ipld_pb::DagPbCodec;
use iroh_car::CarHeader;
use iroh_car::*;
use multihash::{Code::Blake2b256, MultihashDigest};
use thiserror::Error;
use unixfs_v1::{PBLink, PBNode, UnixFs, UnixFsType};

const MAX_CAR_SIZE: usize = 104752742; // 99.9mb

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
trait ToVec {
    fn to_vec(&self) -> Vec<u8>;
}
impl<'a> ToVec for PBNode<'a> {
    fn to_vec(&self) -> Vec<u8> {
        use quick_protobuf::message::MessageWrite;
        use quick_protobuf::Writer;

        let mut ret = vec![];
        let mut writer = Writer::new(&mut ret);
        self.write_message(&mut writer).unwrap();
        ret
    }
}
impl<'a> ToVec for UnixFs<'a> {
    fn to_vec(&self) -> Vec<u8> {
        use quick_protobuf::message::MessageWrite;
        use quick_protobuf::Writer;

        let mut ret = vec![];
        let mut writer = Writer::new(&mut ret);
        self.write_message(&mut writer).unwrap();
        ret
    }
}

fn fake_root() -> (Cid, Vec<u8>) {
    let data_bytes = UnixFs {
        Type: UnixFsType::Raw,
        Data: None,
        filesize: None,
        blocksizes: vec![],
        hashType: None,
        fanout: None,
        mode: None,
        mtime: None,
    }
    .to_vec();

    let node_bytes = PBNode {
        Links: vec![],
        Data: Some(Cow::from(data_bytes)),
    }
    .to_vec();

    let digest = Blake2b256.digest(&node_bytes);
    let cid = Cid::new_v1(DagPbCodec.into(), digest);
    (cid, node_bytes)
}

pub struct Car<'a, W: io::Write> {
    buf: Vec<u8>,
    block_size: usize,
    chunks: Vec<(Cid, Vec<u8>)>,
    links: Vec<PBLink<'a>>,
    blocksizes: Vec<u64>,
    fake_root: (Cid, Vec<u8>),
    name: String,
    next_writer: W,
}

impl<'a, W: io::Write> Car<'a, W> {
    pub fn new(name: String, block_size: usize, next_writer: W) -> Car<'a, W> {
        Car {
            buf: Vec::with_capacity(block_size * 2),
            block_size,
            chunks: Vec::with_capacity(MAX_CAR_SIZE / block_size),
            links: vec![],
            blocksizes: vec![],
            fake_root: fake_root(),
            name,
            next_writer,
        }
    }

    fn chunks_extend(&mut self, chunk: (Cid, Vec<u8>), root: Option<(Cid, Vec<u8>)>) -> Result<Option<Vec<u8>>, Error> {
        self.chunks.push(chunk);
        let max_chunk_size = MAX_CAR_SIZE / self.block_size;

        if root.is_some() || self.chunks.len() == max_chunk_size {
            let chunks = mem::replace(
                &mut self.chunks,
                Vec::with_capacity(max_chunk_size),
            );

            let (root_cid, root_bytes) = match root { Some(x) => x, None => self.fake_root.clone()};
            let header = CarHeader::new(vec![root_cid]);

            let mut buffer = Vec::with_capacity(MAX_CAR_SIZE);
            let mut writer = CarWriter::new(header, &mut buffer);
            for (cid, bytes) in chunks {
                writer.write(cid, bytes)?;
            }
            writer.write(root_cid, root_bytes)?;
            writer.finish()?;

            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }

    fn buf_to_chunk(&mut self) -> Result<Option<Vec<u8>>, Error> {
        let mut buf = mem::replace(&mut self.buf, vec![]);
        if buf.len() > self.block_size {
            self.buf = buf.split_off(self.block_size);
        }

        let digest = Blake2b256.digest(&buf);
        let cid = Cid::new_v1(0x55, digest);
        let size = buf.len() as u64;
        let link = PBLink {
            Name: None,
            Hash: Some(cid.to_bytes().into()),
            Tsize: Some(size),
        };
        self.blocksizes.push(size);
        self.links.push(link);
        self.chunks_extend((cid, buf), None)
    }

    fn buf_extend(&mut self, buf: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.buf.extend(buf);

        if self.buf.len() >= self.block_size {
            self.buf_to_chunk()
        } else {
            Ok(None)
        }
    }

    fn links_to_file(&mut self) -> (Cid, Vec<u8>, u64) {
        let links = mem::replace(&mut self.links, vec![]);
        let blocksizes = mem::replace(&mut self.blocksizes, vec![]);

        let filesize = blocksizes.iter().sum::<u64>();
        let data_bytes = UnixFs {
            Type: UnixFsType::File,
            Data: None,
            filesize: Some(filesize),
            blocksizes,
            hashType: None,
            fanout: None,
            mode: None,
            mtime: None,
        }
        .to_vec();

        let node_bytes = PBNode {
            Links: links,
            Data: Some(Cow::from(data_bytes)),
        }
        .to_vec();

        let digest = Blake2b256.digest(&node_bytes);
        let cid = Cid::new_v1(DagPbCodec.into(), digest);
        let node_len = node_bytes.len() as u64;
        (cid, node_bytes, node_len + filesize)
    }

    fn gen_root(&mut self, files: Vec<(Cid, u64)>) -> (Cid, Vec<u8>) {
        let data_bytes = UnixFs {
            Type: UnixFsType::Directory,
            Data: None,
            filesize: None,
            blocksizes: vec![],
            hashType: None,
            fanout: None,
            mode: None,
            mtime: None,
        }
        .to_vec();

        let links = files
            .iter()
            .map(|(cid, total_size)| PBLink {
                Name: Some(Cow::from(self.name.as_str())),
                Hash: Some(Cow::from(cid.to_bytes())),
                Tsize: Some(*total_size),
            })
            .collect::<Vec<_>>();

        let node_bytes = PBNode {
            Links: links,
            Data: Some(Cow::from(data_bytes)),
        }
        .to_vec();

        let digest = Blake2b256.digest(&node_bytes);
        let cid = Cid::new_v1(DagPbCodec.into(), digest);

        (cid, node_bytes)
    }

}

impl<'a, W: io::Write> io::Write for Car<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = self.buf_extend(buf)?;

        // this scope is for memory release for car(Vec<u8>)
        let need_flush = if let Some(car) = result {
            self.next_writer().write(&car)? == 0
        } else {
            false
        };

        if need_flush {
            self.next_writer().flush()?;
        }

        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        // flush the remaining buf
        let result = self.buf_to_chunk()?;
        if let Some(car) = result {
            self.next_writer().write(&car)?;
        }

        // combine raw chunks to a file PBNode
        let (file_cid, file_pb_bytes, total_size) = self.links_to_file();
        // generate root PBNode
        let root = self.gen_root(vec![(file_cid, total_size)]);

        // flush the remaining chunks
        let result = self.chunks_extend((file_cid, file_pb_bytes), Some(root))?;
        if let Some(car) = result {
            self.next_writer().write(&car)?;
        }

        self.next_writer().flush()?;
        Ok(())
    }
}

impl<'a, W: io::Write> ChainWrite<W> for Car<'a, W> {
    fn next(self) -> W {
        self.next_writer
    }

    fn next_writer(&mut self) -> &mut W {
        &mut self.next_writer
    }
}
