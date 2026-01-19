use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::crypto;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Aes256(crypto::aes256::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match &self {
            Self::Io(e) => e.to_string(),
            Self::Aes256(e) => e.to_string(),
        };

        write!(f, "{}", res)
    }
}

pub fn get_dir() -> PathBuf {
    dirs::config_dir()
        .map(|v| v.join("Wolfyxon/dove"))
        .unwrap_or(Path::new("dove").to_path_buf())
}

pub fn get_token_file_path() -> PathBuf {
    get_dir().join("DO_NOT_SHARE.dat")
}

fn get_encrypted_token() -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(get_token_file_path())?;
    let mut buf: Vec<u8> = Vec::new();

    file.read_to_end(&mut buf)?;

    Ok(buf)
}

pub fn get_token() -> Result<String, Error> {
    let encrypted = get_encrypted_token().map_err(|e| Error::Io(e))?;
    crypto::aes256::decrypt_string(encrypted).map_err(|e| Error::Aes256(e))
}

fn save_encrypted_token(buf: &mut Vec<u8>) -> Result<(), Error> {
    let mut file = File::create(get_token_file_path()).map_err(|e| Error::Io(e))?;
    file.write_all(buf);

    Ok(())
}

pub fn save_token(token: String) -> Result<(), Error> {
    let mut encrypted = crypto::aes256::encrypt_string(token).map_err(|e| Error::Aes256(e))?;

    save_encrypted_token(&mut encrypted)
}
