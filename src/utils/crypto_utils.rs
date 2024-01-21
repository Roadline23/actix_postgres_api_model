use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use dotenv::dotenv;
use generic_array::GenericArray;
use rand::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;
use tracing::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub order: [u8; 16],
    pub content: Vec<u8>,
}

pub fn encrypt_payload<T: Serialize>(payload: &T) -> Result<EncryptedPayload, ()> {
    dotenv().ok();
    let encryption_key: String = env::var("ENCRYPTION_KEY").expect("Failed to load env var");
    let key = GenericArray::from_slice(encryption_key.as_bytes());

    let cipher = Aes256Gcm::new(&key);

    let iv: [u8; 12] = rand::thread_rng().gen();
    let iv_bonus: [u8; 4] = rand::thread_rng().gen();
    let insert_indexes = [2, 5, 8, 10];
    let mut combined_iv = [0; 16];

    for i in 0..16 {
        if insert_indexes.contains(&i) {
            combined_iv[i] = iv_bonus[insert_indexes.iter().position(|&x| x == i).unwrap()];
        } else {
            combined_iv[i] = iv[i - insert_indexes.iter().filter(|&&x| x < i).count()];
        }
    }

    let nonce = aes_gcm::Nonce::from_slice(&iv);

    let text = serde_json::to_string(payload);

    if text.is_err() {
        error!(
            "Cannot serialize payload on encryption: {}",
            text.err().unwrap()
        );
        return Err(());
    }

    let cipher_text = match cipher.encrypt(&nonce, text.unwrap().as_bytes()) {
        Ok(c) => c,
        Err(err) => {
            error!("Cannot encrypt payload: {}", err);
            return Err(());
        }
    };

    let encrypted_payload = EncryptedPayload {
        order: combined_iv,
        content: cipher_text,
    };
    Ok(encrypted_payload)
}

pub fn decrypt_payload<T: for<'a> DeserializeOwned>(
    iv: &[u8],
    cipher_text: &[u8],
) -> Result<T, ()> {
    dotenv().ok();
    let encryption_key: String = env::var("ENCRYPTION_KEY").expect("Failed to load env var");
    let cipher = match Aes256Gcm::new_from_slice(encryption_key.as_bytes()) {
        Ok(c) => c,
        Err(err) => {
            error!("Cannot create cipher: {}", err);
            return Err(());
        }
    };

    let mut restored_iv = [0; 12];
    let insert_indexes = [2, 5, 8, 10];
    let mut restored_iv_index = 0;

    for i in 0..16 {
        if !insert_indexes.contains(&i) {
            restored_iv[restored_iv_index] = iv[i];
            restored_iv_index += 1;
        }
    }

    let nonce = aes_gcm::Nonce::from_slice(&restored_iv);
    match cipher.decrypt(&nonce, cipher_text) {
        Ok(decrypted_text) => match serde_json::from_slice(&decrypted_text) {
            Ok(payload) => Ok(payload),
            Err(err) => {
                error!("Cannot deserialize payload: {}", err);
                return Err(());
            }
        },
        Err(err) => {
            error!("ALERT Cannot decrypt payload: {}", err);
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_encrypt_and_decrypt_data() {
        let payload: String = String::from("Hello World!");
        let encrypted_payload = encrypt_payload(&"Hello World!").unwrap();
        let decrypted_payload: String =
            decrypt_payload(&encrypted_payload.order, &encrypted_payload.content).unwrap();
        assert_eq!(&payload, &decrypted_payload);
    }
}
