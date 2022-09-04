[![w3s crate](https://img.shields.io/crates/v/w3s?style=flat-square)](https://crates.io/crates/w3s)
[![w3s doc](https://img.shields.io/docsrs/w3s?style=flat-square)](https://docs.rs/w3s/latest/w3s/)

# w3s
A Rust crate to the easily upload file or directory to Web3.Storage with optional encryption and compression.

## Usage
Add the following line to your Cargo.toml file:
```
w3s = { version = "0.2", features = ["all"] }
```

## Example

 To upload a single file:
 ```rust
  let cid_result = w3s::helper::upload(
     path,  // the file path 
     auth_token,  // the api token created in web3.storage
     2,  // max concurrent upload threads
     Some(Arc::new(Mutex::new(|name, part, pos, total| {  // the progress listener
         println!("name: {name} part:{part} {pos}/{total}");
     }))),
     Some(None),  // if packed in CAR with custom block size, `Some(None)` means packed in CAR with default 256K block size
     Some(&mut b"abcd1234".to_owned()),  // if use encryption with password
     Some(None),  // if use compression with zstd level, `Some(None)` means uses compression with zstd level at 10
 )
 .await?;
 ```
 
 To upload a directory:
 ```rust
 let cid_result = w3s::helper::upload_dir(
     path,  // the folder path
     None,  // file filter which can bypass specific files
     auth_token,  // the api token created in web3.storage
     2,  // max concurrent upload threads
     Some(Arc::new(Mutex::new(|name, part, pos, total| {  // the progress listener
         println!("name: {name} part:{part} {pos}/{total}");
     }))),
     None,  // if use encryption with password
     None,  // if use compression with zstd level
 )
 .await?;
 ```
 
 To download a compressed and encrypted file from IPFS gateway:
 ```rust
 w3s::helper::download(
     url,  // the whole url pointing to the file under the IPFS geteway
     name,  // just a label that will later be passed to the progress listener
     &mut file,  // file to written
     Some(Arc::new(Mutex::new(|name, _, pos, total| {  // the progress listener
         println!("name: {name} {pos}/{total}");
     }))),
     None,  // start offset which should be `None` for compressed or encrypted file
     Some(b"abcd1234".to_vec()),  // use decryption with password
     true,  // use decompression
 )
 .await?;
 ```

## Details about how to use
Please check the [examples/](examples/) folder for different usage examples.

## TODO
- [x] Composable chain writer
- [x] Gateways availability checker
- [x] Single file upload though `api.web3.storage/upload`
- [x] Single CAR upload though `api.web3.storage/car`
- [x] Single file/CAR concurrent uploads
- [x] Single file/CAR upload with compression
- [x] Single file/CAR upload with encryption
- [x] Auto split for >100MB single file upload
- [x] CAR generation from single file
- [x] Single file/car download
- [x] Single file/car download with decompression and decryption
- [x] Directory upload by CAR writer
- [x] Directory upload with compression and encryption
- [x] Code comments
- [x] Documentation

## Chain writer
The w3s crate contains several writers for the upload tasks. You can put writers according to your needs.

For example, if you'd like to compress and encrypt your data before upload:
```
Get the file -> compression writer -> encryption writer -> split writer -> upload writer
```

If you'd like to upload a CAR file:
```
Get the file -> CAR writer -> upload writer
```

If you'd like to encrypt the file before CAR it and upload:
```
Get the file -> encryption writer -> CAR writer -> upload writer
```
