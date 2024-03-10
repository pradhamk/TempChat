use std::collections::HashMap;

use crate::structs::{EncData, Error, Join, JoinMessage, KeyMessage};
use aes_siv::{Aes256SivAead, aead::KeyInit};
use futures_util::stream::SplitSink;
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug)]
pub struct Client {
    pub username: String,
    pub write: SplitSink<WebSocketStream<TcpStream>, Message>,
    pub registered: bool,
    pub pub_key: Option<RsaPublicKey>,
    pub priv_key: Option<RsaPrivateKey>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SendData {
    Error(Error),
    Shutdown,
    JoinMessage(JoinMessage),
    KeyMessage(KeyMessage),
    EncData(EncData)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RecvData {
    EncData(EncData),
    Join(Join),
    Exit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Exit {
    pub username: String,
}

pub struct ChatData {
    pub key_cipher: Aes256SivAead,
    pub key: Vec<u8>,
    pub peer_map: HashMap<String, Client>,
    pub user_limit: i32,
    pub host_username: String,
}

impl Default for ChatData {
    fn default() -> Self {
        ChatData {
            key_cipher: Aes256SivAead::new(&Aes256SivAead::generate_key(&mut OsRng)),
            key: Vec::new(),
            peer_map: HashMap::new(),
            user_limit: 2,
            host_username: String::new()
        }
    }
}