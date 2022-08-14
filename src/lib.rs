//! w3s is a crate to handle <https://web3.storage/> API.
//! 
//! These operations are supported.
//! * Uploads files with encryption and compression.
//! * CAR file uploading is supported.
//! * Checks uploaded files.
//! * Checks IPFS gateways availability.
//! * Downloads uploaded file with auto merge, decryption and decompression.
//! 
//! ## Feature flags
//! * `encryption`: Enables encryption during the uploading process and decryption during the downloading process.
//! * `compression`: Enables compression during the uploading process and decompression during the downloading process.
//! * `all`: Enables all the features listed above.
//! 

pub mod api;
pub mod gateway;
pub mod writer;
pub mod helper;

/// This module is from [https://github.com/n0-computer/iroh](https://github.com/n0-computer/iroh).
pub mod iroh_car;
