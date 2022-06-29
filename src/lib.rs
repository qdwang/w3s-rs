//! w3s is a crate to handle <https://web3.storage/> API.
//! 
//! These operations are supported.
//! 1. Uploads one file with size split, encryption and compression.
//! 2. Checks uploaded files.
//! 3. Checks IPFS gateways availability.
//! 4. Downloads uploaded file with auto merge, decryption and decompression.
//! 
//! ## Feature flags
//! * `encryption`: Enables encryption during the uploading process and decryption during the downloading process.
//! * `compression`: Enables compression during the uploading process and decompression during the downloading process.
//! * `all`: Enables all the features listed above.
//! 

pub mod gateway;