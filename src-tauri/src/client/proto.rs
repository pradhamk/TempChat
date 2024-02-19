use serde::{Deserialize, Serialize};
use crate::structs::{BroadcastMessage, Error, JoinMessage, UserMessage, Join};

#[derive(Deserialize, Serialize, Debug)]
pub enum SendData {
    Join(Join),
    UserMessage(UserMessage),
    Exit
}

#[derive(Deserialize, Serialize, Debug)]
pub enum RecvData {
    BroadcastMessage(BroadcastMessage),
    Error(Error),
    Shutdown,
    JoinMessage(JoinMessage)
}