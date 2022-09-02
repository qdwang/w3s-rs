//! `w3s` is a crate to help users to upload file and directory to <https://web3.storage/> more easily.
//!
//! These features are supported:
//! * web3.storage API accessing.
//! * Uploads files with encryption and compression.
//! * CAR file uploading is supported.
//! * Checks uploads though IPFS gateways checker.
//! * Downloads uploaded file with auto decryption and decompression.
//!
//! ## Feature flags
//! * `encryption`: Enables encryption during the uploading process and decryption during the downloading process.
//! * `zstd`: Enables compression during the uploading process and decompression during the downloading process.
//! * `all`: Enables all the features listed above.
//!
//! ## Example
//!
//! To upload a single file:
//! ```rust
//!  let cid_result = w3s::helper::upload(
//!     path,  // the file path 
//!     auth_token,  // the api token created in web3.storage
//!     2,  // max concurrent upload threads
//!     Some(Arc::new(Mutex::new(|name, part, pos, total| {  // the progress listener
//!         println!("name: {name} part:{part} {pos}/{total}");
//!     }))),
//!     Some(None),  // if packed in CAR with custom block size, `Some(None)` means packed in CAR with default 256K block size
//!     Some(&mut b"abcd1234".to_owned()),  // if use encryption with password
//!     Some(None),  // if use compression with zstd level, `Some(None)` means uses compression with zstd level at 10
//! )
//! .await?;
//! ```
//! 
//! To upload a directory:
//! ```rust
//! let cid_result = w3s::helper::upload_dir(
//!     path,  // the folder path
//!     None,  // file filter which can bypass specific files
//!     auth_token,  // the api token created in web3.storage
//!     2,  // max concurrent upload threads
//!     Some(Arc::new(Mutex::new(|name, part, pos, total| {  // the progress listener
//!         println!("name: {name} part:{part} {pos}/{total}");
//!     }))),
//!     None,  // if use encryption with password
//!     None,  // if use compression with zstd level
//! )
//! .await?;
//! ```
//! 
//! To download a compressed and encrypted file from IPFS gateway:
//! ```rust
//! w3s::helper::download(
//!     url,  // the whole url pointing to the file under the IPFS geteway
//!     name,  // just a label that will later be passed to the progress listener
//!     &mut file,  // file to written
//!     Some(Arc::new(Mutex::new(|name, _, pos, total| {  // the progress listener
//!         println!("name: {name} {pos}/{total}");
//!     }))),
//!     None,  // start offset which should be `None` for compressed or encrypted file
//!     Some(b"abcd1234".to_vec()),  // use decryption with password
//!     true,  // use decompression
//! )
//! .await?;
//! ```

pub mod api;
pub mod gateway;
pub mod helper;
pub mod writer;

/// This module is from [https://github.com/n0-computer/iroh](https://github.com/n0-computer/iroh).
#[allow(dead_code)]
pub(crate) mod iroh_car;
