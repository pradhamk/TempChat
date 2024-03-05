use crate::server::proto::{Client, Exit, RecvData, SendData, ChatData};
use crate::structs::{BroadcastMessage, EncData, Error, Join, JoinMessage, KeyMessage, UserMessage};
use aes_gcm::aead::Aead;
use aes_gcm::{AeadCore, Aes256Gcm, KeyInit};
use chrono::Local;
use futures_util::stream::SplitStream;
use futures_util::{lock::Mutex, stream::StreamExt, SinkExt};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use std::{collections::HashMap, sync::Arc};
use tauri::Window;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message::Text};
use uuid::Uuid;

pub static USERNAME: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new(String::new())));
pub static PEER_MAP: Lazy<Mutex<HashMap<String, Client>>> = Lazy::new(|| Mutex::new(HashMap::new()));
pub static USER_LIMIT: Lazy<Arc<Mutex<i32>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
pub static CHAT_DATA: Lazy<Arc<Mutex<ChatData>>> = Lazy::new(|| Arc::new(Mutex::new(ChatData::default())));

pub async fn handle_connection(
    stream: TcpStream,
) -> Result<Option<(SplitStream<WebSocketStream<TcpStream>>, String)>, serde_json::Error> {
    println!("New client connected: {:#?}", stream.peer_addr());
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
        let (write, mut read) = ws_stream.split();
        let uid = Uuid::new_v4().to_string();

        println!("Assigning client with uid {}", uid);
        let client = Client {
            username: "".into(),
            write,
            registered: false,
            pub_key: None,
            priv_key: None,
        };

        PEER_MAP.lock().await.insert(uid.clone(), client);
        return Ok(Some((read, uid)))
    }
    Ok(None)
}

pub async fn handle_message(
    message: &RecvData,
    window: &Window,
    uid: &str,
) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    match message {
        RecvData::EncData(enc_data) => {
            if !registered(&uid).await {
                if let Err(_err) = send_err(&uid, "User must be registered".into()).await {
                    close_client(&uid).await;
                }
                return Ok(());
            }

            if let Ok(msg_data) = decrypt_data(enc_data).await {
                if let Ok(message_data) = serde_json::from_str(&String::from_utf8(msg_data).unwrap()) {
                    if let Err(err) = handle_user_message(&message_data, Some(&uid), &window).await {
                        println!("Error handling user message: {:?}", err);
                        close_client(&uid).await;
                        return Err(err);
                    }
                }
            }
        }
        RecvData::Join(join_data) => {
            if let Err(err) = handle_join(join_data, &uid, &window).await {
                println!("Error handling join: {:?}", err);
                close_client(&uid).await;
                return Err(err);
            }
        }
        RecvData::Exit => {
            if registered(&uid).await {
                window
                    .emit(
                        "client_exit",
                        serde_json::to_string(&Exit {
                            username: get_username(&uid).await,
                        })
                        .unwrap(),
                    )
                    .unwrap();
            }
            let client_res = remove_client(&uid).await;
            if let Some(mut client) = client_res {
                let _ = client.write.close().await;
            }
        }
    };
    Ok(())
}

async fn decrypt_data(enc_data: &EncData) -> Result<Vec<u8>, aes_gcm::Error>{
    let key_cipher = &CHAT_DATA.lock().await.key_cipher;
    key_cipher.decrypt(enc_data.nonce.as_slice().into(), enc_data.data.as_slice())
}

async fn remove_client(uid: &str) -> Option<Client> {
    let mut clients = PEER_MAP.lock().await;
    clients.remove(uid)
}

async fn registered(uid: &str) -> bool {
    let clients = PEER_MAP.lock().await;
    clients.get(uid).map_or(false, |client| client.registered)
}

async fn get_joined() -> i32 {
    let clients = PEER_MAP.lock().await;
    let mut joined = 0;
    for client in clients.values() {
        if client.registered {
            joined += 1;
        }
    }
    joined
}

async fn get_username(uid: &str) -> String {
    let clients = PEER_MAP.lock().await;
    clients
        .get(uid)
        .map_or_else(|| "".to_string(), |client| client.username.clone())
}

pub async fn handle_user_message(
    message: &UserMessage,
    uid: Option<&str>,
    window: &Window,
) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let chat_data = CHAT_DATA.lock().await; 

    let send_data = BroadcastMessage {
        sender: if uid.is_some() {
            get_username(uid.unwrap()).await
        } else {
            USERNAME.lock().await.clone()
        },
        content: message.content.clone(),
        created: Local::now().format("%H:%M:%S").to_string(),
    };
    let string_data = serde_json::to_string(&send_data).expect("Couldn't convert message to string");

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let cipher_text = chat_data.key_cipher.encrypt(&nonce, string_data.as_bytes()).expect("Couldn't encrypt message data");

    let enc_data = serde_json::to_string(&SendData::EncData(EncData {
        nonce: nonce.to_vec(),
        data: cipher_text
    })).expect("Couldn't convert encrypted message to string");

    broadcast(&enc_data).await;
    window.emit("new-message", &string_data).unwrap();
    Ok(())
}

async fn broadcast(message: &str) {
    let mut clients = PEER_MAP.lock().await;

    for client in clients.values_mut() {
        if !client.registered {
            continue;
        }
        if let Err(err) = client.write.send(Text(message.to_string())).await {
            println!("Error broadcasting message: {:?}", err);
        }
    }
}

async fn handle_join(
    join_data: &Join,
    uid: &str,
    window: &Window,
) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let limit = *USER_LIMIT.lock().await;

    println!("Joined");

    if get_joined().await + 1 > limit {
        if let Err(_err) = send_err(&uid, "Max joins reached".into()).await {
            close_client(&uid).await;
        }
        return Ok(());
    }

    let try_pub_key = RsaPublicKey::from_pkcs1_pem(&join_data.pub_key); 
    if try_pub_key.is_err() {
        if let Err(_err) = send_err(&uid, "Invalid public key provided".into()).await {
            close_client(&uid).await;
        }
        return Ok(());
    }

    let mut clients = PEER_MAP.lock().await;

    for client in clients.values() {
        if &client.username == &join_data.username {
            if let Err(_err) = send_err(&uid, "Username already taken".into()).await {
                close_client(&uid).await;
            }
            return Ok(());
        }
    }

    let client = clients
        .get_mut(uid)
        .ok_or(tokio_tungstenite::tungstenite::Error::AlreadyClosed)?;
    if client.username.len() > 15 {
        if let Err(_err) = send_err(&uid, "Username too long".into()).await {
            close_client(&uid).await;
        }
        return Ok(());
    }
    client.username = join_data.username.clone();
    client.registered = true;
    client.pub_key = Some(try_pub_key.unwrap());

    println!("Client Username: {}", client.username);

    let join_broadcast = serde_json::to_string(&SendData::JoinMessage(JoinMessage {
        joined: client.username.clone(),
    }))
    .unwrap();

    let chat_key = &CHAT_DATA.lock().await.key;
    let enc_key = client.pub_key.as_mut().unwrap().encrypt(&mut OsRng, Pkcs1v15Encrypt, &chat_key).expect("Couldn't encrypt chat key with user public key");

    let key_msg = serde_json::to_string(&SendData::KeyMessage(KeyMessage {
        key: enc_key        
    }))
    .unwrap();

    client.write.send(Text(key_msg)).await?;

    drop(clients);
    broadcast(&join_broadcast).await;

    window.emit("join", join_broadcast).unwrap();

    Ok(())
}

async fn send_err(uid: &str, message: String) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let mut clients = PEER_MAP.lock().await;
    if let Some(client) = clients.get_mut(uid) {
        let error = serde_json::to_string(&SendData::Error(Error { error_msg: message }))
            .expect("Couldn't convert error message");
        client.write.send(Text(error)).await
    } else {
        Err(tokio_tungstenite::tungstenite::Error::AlreadyClosed)
    }
}

pub async fn close_client(uid: &str) {
    println!("Closing client {}", uid);
    if let Some((_, mut client)) = PEER_MAP.lock().await.remove_entry(uid) {
        if let Err(err) = client.write.close().await {
            println!("Error closing client socket: {:?}", err);
        }
    }
}

pub async fn chat_shutdown() {
    let mut clients = PEER_MAP.lock().await;
    for client in clients.values_mut() {
        let _ = client
            .write
            .send(Text(serde_json::to_string(&SendData::Shutdown).unwrap()))
            .await;
        if let Err(err) = client.write.close().await {
            println!("Error closing client socket: {:?}", err);
        }
    }
    clients.clear();
}
