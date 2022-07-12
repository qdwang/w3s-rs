use aead::generic_array::{ArrayLength, GenericArray};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20::XChaCha20;
use poly1305::universal_hash::{NewUniversalHash, UniversalHash};
use poly1305::{Key, Poly1305};
use rand::prelude::*;
use thiserror::Error;
use zeroize::Zeroize;

const SALT_SIZE: usize = 8;
const NONCE_SIZE: usize = 24;
const BLOCK_SIZE: u64 = 64;

pub struct Cipher {
    cipher: XChaCha20,
    mac: Poly1305,
    encrypted_len: usize,
    salt: [u8; SALT_SIZE],
    nonce: [u8; NONCE_SIZE],
}

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
}

impl Cipher {
    pub fn new(pwd: &[u8]) -> Result<Cipher, Error> {
        let mut rng = rand::thread_rng();
        let mut salt = [0u8; SALT_SIZE];
        let mut nonce = [0u8; NONCE_SIZE];
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce);

        Self::new_with_details(pwd, salt, nonce)
    }

    fn new_with_details(
        pwd: &[u8],
        salt: [u8; SALT_SIZE],
        nonce: [u8; NONCE_SIZE],
    ) -> Result<Cipher, Error> {
        let salt_string = SaltString::b64_encode(&salt).map_err(|e| Error::PasswordHashError(e))?;

        let hashed_pwd = Argon2::default()
            .hash_password(pwd, &salt_string)
            .map_err(|e| Error::PasswordHashError(e))?
            .hash
            .ok_or(Error::HashResultError)?;

        let key = Key::from_slice(hashed_pwd.as_bytes());
        let mut cipher = XChaCha20::new(key.into(), &nonce.into());

        // init for mac
        let mut mac_key = poly1305::Key::default();
        cipher.apply_keystream(&mut *mac_key);

        let mac = Poly1305::new(GenericArray::from_slice(&*mac_key));
        mac_key.zeroize();

        cipher.seek(BLOCK_SIZE);

        Ok(Cipher {
            cipher,
            mac,
            encrypted_len: 0,
            salt,
            nonce,
        })
    }

    pub fn encrypt(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut buffer = bytes.to_vec();
        self.cipher.apply_keystream(&mut buffer);
        self.mac.update_padded(&buffer);
        self.encrypted_len += buffer.len();
        buffer
    }

    pub fn finalize_mac(&mut self) -> Vec<u8> {
        let mut block = GenericArray::default();
        block[..8].copy_from_slice(&0u64.to_le_bytes());
        block[8..].copy_from_slice(&(self.encrypted_len as u64).to_le_bytes());

        self.mac.update(&block);
        self.mac.to_owned().finalize().into_bytes().to_vec()
    }
}
