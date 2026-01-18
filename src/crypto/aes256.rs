use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::{Aead, generic_array::sequence::GenericSequence}, aes::Aes256};

// TODO: Figure out how to handle nonce properly

fn get_key() -> Result<Aes256Gcm, String> {
    // TODO: Generate key based on runtime properties
    eprintln!("WARNING: Encryption key not properly generated");

    let key_slice: &[u8] = &[42; 32];
    let key_array = Key::<Aes256Gcm>::from_slice(key_slice);
    Ok(Aes256Gcm::new(key_array))
}

pub fn encrypt_string(plaintext: String) -> Result<Vec<u8>, String> {
    let key = get_key()?;
    let nonce = Nonce::generate(|i| 0);
    
    key.encrypt(&nonce, plaintext.as_bytes()).map_err(|e| e.to_string())
}

pub fn decrypt_string(cipher: Vec<u8>) -> Result<String, String> {
    let buf = decrypt(cipher)?;

    String::from_utf8(buf).map_err(|e| e.to_string())
}

pub fn decrypt(cipher: Vec<u8>) -> Result<Vec<u8>, String> {
    let key = get_key()?;
    let nonce = Nonce::generate(|i| 0);

    key.decrypt(&nonce, cipher.as_ref()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use crate::crypto::{decrypt_string, encrypt_string};

    #[test]
    fn test_encrypt_decrypt() {
        let plain = "Hello there 123 .-_?/".to_string();
        let encrypted = encrypt_string(plain.to_owned()).unwrap();
        let decrypted = decrypt_string(encrypted).unwrap();

        assert_eq!(plain, decrypted);
    }
}
