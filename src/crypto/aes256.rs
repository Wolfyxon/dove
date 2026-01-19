use std::{fmt::Display, string::FromUtf8Error};

use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, generic_array::sequence::GenericSequence},
};
use rand::RngCore;

use crate::crypto;

#[derive(Debug)]
pub enum Error {
    // TODO: Add error for get_key()
    Lib(aes_gcm::Error),
    FromUtf8(FromUtf8Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match &self {
            Self::Lib(e) => e.to_string(),
            Self::FromUtf8(e) => e.to_string(),
        };

        write!(f, "{}", res)
    }
}

fn get_key() -> Result<Aes256Gcm, Error> {
    // Returns Result in case this can error in the future
    let mut rng = crypto::get_machine_id_rng();
    let mut key_slice: [u8; 32] = [0; 32];

    rng.fill_bytes(&mut key_slice);

    let key = Key::<Aes256Gcm>::from_slice(&key_slice);

    Ok(Aes256Gcm::new(&key))
}

pub fn encrypt_string(plaintext: String) -> Result<Vec<u8>, Error> {
    let key = get_key()?;
    let nonce = Nonce::generate(|_| 0);

    key.encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| Error::Lib(e))
}

pub fn decrypt_string(cipher: Vec<u8>) -> Result<String, Error> {
    let buf = decrypt(cipher)?;

    String::from_utf8(buf).map_err(|e| Error::FromUtf8(e))
}

pub fn decrypt(cipher: Vec<u8>) -> Result<Vec<u8>, Error> {
    let key = get_key()?;
    let nonce = Nonce::generate(|_| 0);

    key.decrypt(&nonce, cipher.as_ref())
        .map_err(|e| Error::Lib(e))
}

#[cfg(test)]
mod tests {
    use crate::crypto::aes256::{decrypt_string, encrypt_string};

    #[test]
    fn test_encrypt_decrypt() {
        let plain = "Hello there 123 .-_?/".to_string();
        let encrypted = encrypt_string(plain.to_owned()).unwrap();
        let decrypted = decrypt_string(encrypted).unwrap();

        assert_eq!(plain, decrypted);
    }
}
