use crate::structs::{
    BroadcastMessage, EncData, Error, Join, JoinMessage, KeyMessage, UserMessage,
};
use futures_util::stream::SplitSink;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SendData {
    Join(Join),
    UserMessage(UserMessage),
    Exit,
    EncData(EncData),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RecvData {
    Error(Error),
    Shutdown,
    JoinMessage(JoinMessage),
    KeyMessage(KeyMessage),
    EncData(EncData),
}

pub struct Client {
    pub write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    pub pub_key: Option<RsaPublicKey>,
    pub priv_key: Option<RsaPrivateKey>,
    pub chat_key: Option<Vec<u8>>,
}

impl Default for Client {
    fn default() -> Self {
        Client {
            write: None,
            pub_key: None,
            priv_key: None,
            chat_key: None,
        }
    }
}
