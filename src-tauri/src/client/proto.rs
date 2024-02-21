use crate::structs::{BroadcastMessage, Error, Join, JoinMessage, UserMessage};
use futures_util::stream::SplitSink;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SendData {
    Join(Join),
    UserMessage(UserMessage),
    Exit,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RecvData {
    BroadcastMessage(BroadcastMessage),
    Error(Error),
    Shutdown,
    JoinMessage(JoinMessage),
}

pub struct Client {
    pub write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}
