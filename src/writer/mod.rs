use std::io;

pub mod car_util;
pub mod car;

pub mod splitter;
pub mod uploader;
pub mod downloader;

#[cfg(feature = "encryption")]
pub mod cipher;

#[cfg(feature = "zstd")]
pub mod decompressor;

pub trait ChainWrite<W: io::Write>: io::Write {
    fn next_writer(&mut self) -> &mut W;
    fn next(self) -> W;
}
