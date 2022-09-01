use aead::generic_array::GenericArray;
use argon2::password_hash::Output;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20::XChaCha20;
use poly1305::universal_hash::{NewUniversalHash, UniversalHash};
use poly1305::Poly1305;
use rand::prelude::*;
use thiserror::Error;
use zeroize::Zeroize;

use super::*;
use std::{io, mem};

const SALT_SIZE: usize = 8;
const NONCE_SIZE: usize = 24;
const BLOCK_SIZE: usize = 64;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The cipher has no hash output.")]
    HashResultError,
    #[error("Argon2 password hash error: {0:?}")]
    PasswordHashError(argon2::password_hash::errors::Error),
    #[error("No enough bytes for salt.")]
    TooShortForSalt,
    #[error("No enough bytes for nonce.")]
    TooShortForNonce,
    #[error("MAC tag error. The encrypted content may has been modified and is not valid.\n{0}")]
    MacTagInvalid(String),
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::Interrupted, e)
    }
}

/// The cipher will pass `[salt][nonce][encrypted data][mac]` to the next writer.
/// All the `[salt][nonce][encrypted data]` parts will be updated into MAC.
///
/// This stream cipher is compatible to the `chacha20poly1305` crate.
/// But if you are using `chacha20poly1305` crate to decrypt the file,
/// you have to use `cipher.decrypt_in_place(nonce, salt + nonce, &mut buffer).unwrap()` to
/// get the correct result.
pub struct Cipher<W: io::Write> {
    cipher: XChaCha20,
    mac: Poly1305,
    kept_key: Output,
    decryption: Option<Vec<u8>>, // Some(password)
    is_prefix_written: bool,
    remnant2mac: Vec<u8>,
    content_len: usize,
    salt: [u8; SALT_SIZE],
    nonce: [u8; NONCE_SIZE],
    next_writer: W,
}

impl<W: io::Write> Cipher<W> {
    pub fn new_decryption(pwd: Vec<u8>, next_writer: W) -> Result<Cipher<W>, Error> {
        let salt = [0u8; SALT_SIZE];
        let nonce = [0u8; NONCE_SIZE];
        let (cipher, mac, kept_key) = Self::gen_cipher_and_mac(&mut [], &salt, &nonce)?;

        Ok(Cipher {
            cipher,
            mac,
            kept_key,
            decryption: Some(pwd),
            is_prefix_written: false,
            remnant2mac: vec![],
            content_len: 0,
            salt,
            nonce,
            next_writer,
        })
    }

    pub fn new(pwd: &mut [u8], next_writer: W) -> Result<Cipher<W>, Error> {
        let mut rng = rand::thread_rng();
        let mut salt = [0u8; SALT_SIZE];
        let mut nonce = [0u8; NONCE_SIZE];
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce);

        let (cipher, mac, kept_key) = Self::gen_cipher_and_mac(pwd, &salt, &nonce)?;

        Ok(Cipher {
            cipher,
            mac,
            kept_key,
            decryption: None,
            is_prefix_written: false,
            remnant2mac: vec![],
            content_len: 0,
            salt,
            nonce,
            next_writer,
        })
    }

    fn reset(&mut self) {
        let (cipher, mac, _) = Self::gen_cipher_and_mac_with_hashed_pwd(self.kept_key, &self.nonce);

        self.mac = mac;
        self.cipher = cipher;
        self.is_prefix_written = false;
        self.remnant2mac = vec![];
        self.content_len = 0;
    }

    fn gen_cipher_and_mac_with_hashed_pwd(
        hashed_pwd: Output,
        nonce: &[u8],
    ) -> (XChaCha20, Poly1305, Output) {
        let mut cipher = XChaCha20::new(hashed_pwd.as_bytes().into(), nonce.into());

        // init for mac
        let mut mac_key = poly1305::Key::default();
        cipher.apply_keystream(&mut *mac_key);

        let mac = Poly1305::new(GenericArray::from_slice(&*mac_key));
        mac_key.zeroize();

        cipher.seek(BLOCK_SIZE);

        (cipher, mac, hashed_pwd)
    }

    fn gen_cipher_and_mac(
        pwd: &mut [u8],
        salt: &[u8],
        nonce: &[u8],
    ) -> Result<(XChaCha20, Poly1305, Output), Error> {
        let salt_string = SaltString::b64_encode(salt).map_err(Error::PasswordHashError)?;

        let hashed_pwd = Argon2::default()
            .hash_password(pwd, &salt_string)
            .map_err(Error::PasswordHashError)?
            .hash
            .ok_or(Error::HashResultError)?;

        pwd.zeroize();

        Ok(Self::gen_cipher_and_mac_with_hashed_pwd(hashed_pwd, nonce))
    }

    fn update_mac(&mut self, buf: &[u8], keep_remnant: bool) {
        let mut buf2mac = mem::take(&mut self.remnant2mac);
        buf2mac.extend(buf);

        if keep_remnant {
            match buf2mac.len() / BLOCK_SIZE {
                0 => self.remnant2mac = buf2mac,
                x => {
                    let index = x * BLOCK_SIZE;
                    self.mac.update_padded(&buf2mac[..index]);
                    self.content_len += index;
                    self.remnant2mac = buf2mac[index..].to_vec();
                }
            }
        } else {
            self.mac.update_padded(&buf2mac);
            self.content_len += buf2mac.len();
        }
    }

    fn encrypt(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut buffer = bytes.to_vec();

        self.cipher.apply_keystream(&mut buffer);
        self.update_mac(&buffer, true);

        buffer
    }
    fn decrypt(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut buffer = bytes.to_vec();

        self.update_mac(&buffer, true);
        self.cipher.apply_keystream(&mut buffer);

        buffer
    }

    pub fn finalize_mac(&mut self) -> Vec<u8> {
        let mut block = GenericArray::default();
        // for associated_data
        block[..8].copy_from_slice(&((SALT_SIZE + NONCE_SIZE) as u64).to_le_bytes());
        // for content data
        block[8..].copy_from_slice(&(self.content_len as u64).to_le_bytes());

        self.mac.update(&block);
        self.mac.to_owned().finalize().into_bytes().to_vec()
    }
}

impl<W: io::Write> io::Write for Cipher<W> {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let buf_size = buf.len();

        if !self.is_prefix_written {
            // add salt and nonce to the start to enable streaming decryption when downloading the file
            let mut prefix = Vec::with_capacity(SALT_SIZE + NONCE_SIZE);

            if let Some(pwd) = self.decryption.as_mut() {
                // this branch is for decryption
                let salt = buf.get(0..SALT_SIZE).ok_or(Error::TooShortForSalt)?;
                let nonce = buf
                    .get(SALT_SIZE..SALT_SIZE + NONCE_SIZE)
                    .ok_or(Error::TooShortForNonce)?;
                let (cipher, mac, _) = Self::gen_cipher_and_mac(pwd, salt, nonce)?;
                pwd.clear();

                self.cipher = cipher;
                self.mac = mac;

                prefix.extend(salt);
                prefix.extend(nonce);

                buf = buf
                    .get(SALT_SIZE + NONCE_SIZE..)
                    .ok_or(Error::TooShortForNonce)?;
            } else {
                prefix.extend(self.salt);
                prefix.extend(self.nonce);

                self.next_mut().write_all(&prefix)?;
            }

            // use salt + nonce as associated_data to update the mac
            self.mac.update_padded(&prefix);

            self.is_prefix_written = true;
        }

        if let Some(raw) = self.decryption.take() {
            if !raw.is_empty() {
                let decrypted = self.decrypt(&raw);
                self.next_mut().write_all(&decrypted)?;
            }
            self.decryption = Some(buf.to_vec())
        } else {
            let encrypted = self.encrypt(buf);
            // since the Upload writer shouldn't be the next one, there is no needs to handle the 0 written length condition.
            self.next_mut().write_all(&encrypted)?;
        }

        Ok(buf_size)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(raw) = self.decryption.take() {
            let (buf, tag) = raw.split_at(raw.len() - 16);
            let decrypted = self.decrypt(buf);
            self.next_mut().write_all(&decrypted)?;
            self.decryption = Some(tag.to_vec());
        }

        self.update_mac(&[], false);

        let mac = self.finalize_mac();

        if let Some(tag) = self.decryption.as_ref() {
            if &mac != tag {
                Err(Error::MacTagInvalid(format!(
                    "calculated: {:02x?}\n from file: {:02x?}",
                    mac, tag
                )))?;
            }
        } else {
            self.next_mut().write_all(&mac)?;
        }

        self.next_mut().flush()?;

        self.reset();

        Ok(())
    }
}

impl<W: io::Write> ChainWrite<W> for Cipher<W> {
    fn next(self) -> W {
        self.next_writer
    }
    fn next_mut(&mut self) -> &mut W {
        &mut self.next_writer
    }
}
