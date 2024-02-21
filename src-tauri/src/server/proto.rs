use crate::structs::{BroadcastMessage, Error, Join, JoinMessage, UserMessage};
use futures_util::stream::SplitSink;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug)]
pub struct Client {
    pub username: String,
    pub write: SplitSink<WebSocketStream<TcpStream>, Message>,
    pub registered: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SendData {
    BroadcastMessage(BroadcastMessage),
    Error(Error),
    Shutdown,
    JoinMessage(JoinMessage),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RecvData {
    UserMessage(UserMessage),
    Join(Join),
    Exit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Exit {
    pub username: String,
}
