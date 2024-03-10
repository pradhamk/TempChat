use aes_siv::{aead::{Aead, OsRng}, Aes256SivAead, Key, KeyInit, Nonce};
use rand::RngCore;

use crate::structs::EncData;

async fn generate_nonce() -> Nonce {
    let mut nonce: [u8; 16] = [0; 16];
    OsRng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce).to_owned()
}

async fn pad_password(password: String) -> Vec<u8> {
    let mut pass_vec = password.as_bytes().to_vec();
    pass_vec.resize(64, 0);
    pass_vec
}

pub async fn create_join_url(tunnel_url: String, password: String) -> Result<String, String> {
    let pass_vec = pad_password(password).await;
    let key: &Key<Aes256SivAead> = pass_vec.as_slice().into();
    let cipher = Aes256SivAead::new(key);
    let nonce = generate_nonce().await;
    let encrypted_url_res = cipher.encrypt(&nonce, tunnel_url.as_bytes());
    match encrypted_url_res {
        Ok((encrypted_url)) => {
            let hex_url = hex::encode(encrypted_url);
            let hex_nonce = hex::encode(nonce);
            Ok(format!("temp://{}_{}", hex_nonce, hex_url))
        },
        Err(aes_siv::Error) => {
            Err("Couldn't encrypt join url".into())
        }
    }
}

pub async fn parse_join_url(join_url: String, password: String) -> Result<String, String> {
    let join_url = join_url.replace("temp://", "");
    let split_url: Vec<&str> = join_url.splitn(2, "_").collect();
    if split_url.len() != 2 {
        return Err("URL is in incorrect format".into())
    }
    let (hex_nonce, hex_url) = (split_url[0], split_url[1]);
    let try_nonce = hex::decode(hex_nonce);
    let try_url = hex::decode(hex_url);
    if try_nonce.is_err() || try_url.is_err() {
        return Err("Could not decode URL".into())
    }
    let nonce = try_nonce.unwrap();
    let nonce = Nonce::from_slice(&nonce);
    let pass_vec = pad_password(password).await;
    let key: &Key<Aes256SivAead> = pass_vec.as_slice().into();
    let cipher = Aes256SivAead::new(key);
    let decrypt_res = cipher.decrypt(&nonce, try_url.unwrap().as_slice());
    match decrypt_res {
        Ok(url) => {
            let parsed_res = String::from_utf8(url);
            if parsed_res.is_err() {
                return Err("Couldn't decrypt URL".into())
            }
            Ok(parsed_res.unwrap())
        },
        Err(aes_siv::Error) => {
            Err("Couldn't decrypt URL".into())
        }
    }
}

pub async fn decrypt_message(enc_data: &EncData, cipher: &Aes256SivAead) -> Result<Vec<u8>, String>{
    let nonce = enc_data.nonce.as_slice();
    let decrypt_res = cipher.decrypt(nonce.into(), enc_data.data.as_slice());
    match decrypt_res {
        Ok(decrypted) => {
            Ok(decrypted)
        },
        Err(_err) => {
            Err("Couldn't decrypt message data".into())
        }
    }
}

pub async fn encrypt_message(message: String, cipher: &Aes256SivAead) -> Result<EncData, String> {
    let nonce = generate_nonce().await;
    let cipher_message_res = cipher.encrypt(&nonce, message.as_bytes());
    match cipher_message_res {
        Ok(cipher_message) => {
            Ok(
                EncData {
                    nonce: nonce.to_vec(),
                    data: cipher_message
                }
            )
        },
        Err(_err) => {
            Err("Couldn't ecnrypt message".into())
        }
    }
}

