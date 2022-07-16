# w3s
A Rust crate to the Web3.Storage API.

## How to use
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
- [ ] Single file/car download
- [ ] Single file/car download with decompression and decryption
- [ ] Directory upload by CAR writer
- [ ] Directory upload with compression and encryption
- [ ] Code comments
- [ ] Documentation

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
