use serde::{Deserialize, Serialize};

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
    pub error_msg: String,
}