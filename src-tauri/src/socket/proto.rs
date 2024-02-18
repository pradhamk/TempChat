use serde::{Deserialize, Serialize};
use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug)]
pub struct Client {
    pub username: String,
    pub write: SplitSink<WebSocketStream<TcpStream>, Message>,
    pub registered: bool
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SendData {
    BroadcastMessage(BroadcastMessage),
    Error(Error),
    Shutdown,
    JoinMessage(JoinMessage)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RecvData {
    UserMessage(UserMessage),
    Join(Join),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Join {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserMessage {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastMessage {
    pub sender: String,
    pub content: String,
    pub created: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinMessage {
    pub joined: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub error_msg: String
}