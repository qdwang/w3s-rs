use super::super::iroh_car;
use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::{borrow::Cow, collections::HashMap, mem};

use cid::Cid;
use ipld_pb::DagPbCodec;
use iroh_car::CarHeader;
use iroh_car::*;
use multihash::{Code::Blake2b256, MultihashDigest};
use quick_protobuf::message::MessageWrite;
use quick_protobuf::Writer;
use unixfs_v1::{PBLink, PBNode, UnixFs, UnixFsType};

pub const MAX_CAR_SIZE: usize = 104752742; // 99.9mb

trait ToVec {
    fn to_vec(&self) -> Vec<u8>;
}
impl<'a> ToVec for PBNode<'a> {
    fn to_vec(&self) -> Vec<u8> {
        let mut ret = vec![];
        let mut writer = Writer::new(&mut ret);
        self.write_message(&mut writer).unwrap();
        ret
    }
}
impl<'a> ToVec for UnixFs<'a> {
    fn to_vec(&self) -> Vec<u8> {
        let mut ret = vec![];
        let mut writer = Writer::new(&mut ret);
        self.write_message(&mut writer).unwrap();
        ret
    }
}

#[derive(Debug)]
pub enum DirectoryItem {
    /// name, path, id
    File(String, String, u64),
    /// name, sub_items
    Directory(String, Vec<DirectoryItem>),
}

impl DirectoryItem {
    pub fn from_path(
        path: &str,
        filter: Option<fn(name: &str, is_file: bool) -> bool>,
    ) -> io::Result<(Vec<Self>, u64)> {
        let path_buf = Path::new(path).to_path_buf();
        let mut id = 0;
        let result = Self::from_path_buf(path_buf, &mut id, filter.unwrap_or(|_, _| true))?;
        Ok((result, id))
    }

    fn from_path_buf(
        path: PathBuf,
        id: &mut u64,
        filter: fn(&str, bool) -> bool,
    ) -> io::Result<Vec<Self>> {
        let dir = fs::read_dir(path)?;
        let mut result = vec![];
        for item in dir {
            let entry = item?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !filter(&name, metadata.is_file()) {
                continue;
            }

            if metadata.is_dir() {
                result.push(Self::Directory(
                    name,
                    Self::from_path_buf(entry.path(), id, filter)?,
                ));
            } else if metadata.is_file() {
                let path = entry.path().to_string_lossy().to_string();
                *id += 1;
                result.push(Self::File(name, path, *id));
            }
        }

        Ok(result)
    }

    pub fn to_unixfs_struct(
        &self,
        id_map: &HashMap<u64, Vec<UnixFsStruct>>,
        collect_blocks: &mut Vec<UnixFsStruct>,
    ) -> UnixFsStruct {
        let block = match self {
            Self::File(name, _, id) => {
                if let Some(blocks) = id_map.get(id) {
                    gen_pbnode_from_blocks(name.clone(), blocks)
                } else {
                    empty_item()
                }
            }
            Self::Directory(name, sub_items) => {
                let items: Vec<UnixFsStruct> = sub_items
                    .iter()
                    .map(|x| x.to_unixfs_struct(id_map, collect_blocks))
                    .collect();
                gen_dir(Some(name.clone()), &items)
            }
        };
        
        collect_blocks.push(block.clone());
        block
    }
}

#[derive(Debug, Clone)]
pub struct UnixFsStruct {
    name: Option<String>,
    cid: Cid,
    data: Vec<u8>,
    size: u64,
}
impl UnixFsStruct {
    pub fn to_link(&self) -> PBLink {
        PBLink {
            Name: None,
            Hash: Some(self.cid.to_bytes().into()),
            Tsize: Some(self.size),
        }
    }
}

/// bafykbzacebrixudpac7a56ypc7lxhwqe5nyvvmyc6mhurq4pc3zmsmymr2cum
fn empty_item() -> UnixFsStruct {
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
    UnixFsStruct {
        name: None,
        cid,
        data: node_bytes,
        size: 0,
    }
}

pub fn gen_car(
    blocks: &mut [UnixFsStruct],
    root_struct: Option<UnixFsStruct>,
) -> Result<Vec<u8>, iroh_car::Error> {
    let root = root_struct.unwrap_or(empty_item());
    let header = CarHeader::new(vec![root.cid]);

    let mut buffer = Vec::with_capacity(MAX_CAR_SIZE);
    let mut writer = CarWriter::new(header, &mut buffer);

    for block in blocks {
        let data = mem::replace(&mut block.data, vec![]);
        writer.write(block.cid, data)?;
    }
    writer.write(root.cid, root.data)?;
    writer.flush()?;

    Ok(buffer)
}

pub fn gen_blocks(buf: Vec<u8>, block_size: usize) -> Vec<UnixFsStruct> {
    buf.chunks(block_size)
        .map(|chunk| {
            let digest = Blake2b256.digest(chunk);
            let cid = Cid::new_v1(0x55, digest);
            UnixFsStruct {
                name: None,
                cid,
                data: chunk.to_vec(),
                size: chunk.len() as u64,
            }
        })
        .collect()
}

pub fn gen_dir(name: Option<String>, items: &[UnixFsStruct]) -> UnixFsStruct {
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

    let mut dir_size = 0;
    let links = items
        .iter()
        .map(|x| {
            dir_size += x.size;
            PBLink {
                Name: x.name.as_ref().map(|x| Cow::from(x)),
                Hash: Some(Cow::from(x.cid.to_bytes())),
                Tsize: Some(x.size),
            }
        })
        .collect::<Vec<_>>();

    let node_bytes = PBNode {
        Links: links,
        Data: Some(Cow::from(data_bytes)),
    }
    .to_vec();

    let digest = Blake2b256.digest(&node_bytes);
    let cid = Cid::new_v1(DagPbCodec.into(), digest);

    let size = dir_size + node_bytes.len() as u64;
    UnixFsStruct {
        name,
        cid,
        data: node_bytes,
        size,
    }
}

pub fn gen_pbnode_from_blocks(name: String, blocks: &[UnixFsStruct]) -> UnixFsStruct {
    let mut filesize = 0u64;
    let (links, blocksizes) = blocks
        .iter()
        .map(|x| {
            filesize += x.size;
            (x.to_link(), x.size)
        })
        .unzip();

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
    let size = node_bytes.len() as u64 + filesize;
    UnixFsStruct {
        name: Some(name),
        cid,
        data: node_bytes,
        size,
    }
}
