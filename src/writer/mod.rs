//! Different writer parts
//! 
use std::io;

pub mod dir;
pub mod car_util;
pub mod car;

pub mod splitter;
pub mod uploader;
pub mod downloader;

#[cfg(feature = "encryption")]
pub mod cipher;

#[cfg(feature = "zstd")]
pub mod decompressor;

/// Describe the trait of writers which can be chained
pub trait ChainWrite<W: io::Write>: io::Write {
    fn next_mut(&mut self) -> &mut W;
    fn next(self) -> W;
}
