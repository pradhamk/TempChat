use std::{error::Error, sync::Arc};

use crate::{
    client::proto::{Client, SendData},
    structs::{BroadcastMessage, EncData, Join},
};
use aes_siv::{aead::{Aead, OsRng}, Aes256SivAead, Key, KeyInit, Nonce};
use futures_util::{lock::Mutex, SinkExt, StreamExt};
use once_cell::sync::Lazy;
use rand::RngCore;
use rsa::{pkcs1::EncodeRsaPublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use tauri::{command, Window};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message::Text};

use super::proto::RecvData;

static CLIENT: Lazy<Arc<Mutex<Client>>> = Lazy::new(|| Arc::new(Mutex::new(Client::default())));

async fn handle_recv_data(mut rx: mpsc::UnboundedReceiver<RecvData>, window: Window) {
    loop {
        let res = rx.recv().await;
        if let Some(recv_data) = res {
            match recv_data {
                RecvData::EncData(enc_data) => {
                    let mut client = CLIENT.lock().await;
                    if client.chat_key.is_some() {
                        let key: &Key<Aes256SivAead> = client.chat_key.as_mut().unwrap().as_slice().into();
                        if let Ok(dec_data) = Aes256SivAead::new(key).decrypt(enc_data.nonce.as_slice().into(), enc_data.data.as_slice()) {
                            if let Ok(broadcast_data) = serde_json::from_str::<BroadcastMessage>(&String::from_utf8(dec_data).expect("Value isn't string")) {
                                println!("Broadcast Data: {:#?}", broadcast_data);
                                window
                                .emit(
                                    "new-message",
                                    serde_json::to_string(&broadcast_data).unwrap(),
                                )
                                .unwrap();
                            }
                        }
                    }
                }
                RecvData::JoinMessage(data) => {
                    window
                        .emit(
                            "join",
                            serde_json::to_string(&RecvData::JoinMessage(data)).unwrap(),
                        )
                        .unwrap();
                }
                RecvData::KeyMessage(msg) => {
                    let enc_key = msg.key;
                    let mut client = CLIENT.lock().await;
                    let dec_data = client.priv_key.as_mut().unwrap().decrypt(Pkcs1v15Encrypt, &enc_key);
                    if dec_data.is_err() {
                        //TODO: Display error to user
                    }
                    let chat_key = dec_data.unwrap();
                    client.chat_key = Some(chat_key);
                }
                RecvData::Error(err) => {
                    window
                        .emit(
                            "error",
                            serde_json::to_string(&RecvData::Error(err)).unwrap(),
                        )
                        .unwrap();
                }
                RecvData::Shutdown => {
                    window.emit("shutdown", {}).unwrap();
                    let mut client = CLIENT.lock().await;
                    let _ = client.write.as_mut().unwrap().close().await;
                    client.write = None;
                }
            }
        }
    }
}


#[command]
pub async fn join_chat(username: String, chat_url: String, password: String, window: Window) -> Result<(), String> {
    let mut rng = rand::rngs::OsRng::default();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Couldn't generate user private key");
    let pub_key = RsaPublicKey::from(&priv_key);

    let chat_url = chat_url.replace("temp://", "");
    let split_url: Vec<&str> = chat_url.splitn(2, '_').collect();
    if split_url.len() != 2 {
        return Err("URL is in incorrect format".to_string());
    }
    let (hex_nonce, hex_url) = (split_url[0], split_url[1]);

    let nonce = hex::decode(hex_nonce).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce);

    let url_res = hex::decode(hex_url).map_err(|e| e.to_string())?;
    let url = String::from_utf8(url_res).map_err(|e| e.to_string())?;

    let mut pass_vec = password.as_bytes().to_vec();
    pass_vec.resize(64, 0);

    let key: &Key<Aes256SivAead> = pass_vec.as_slice().into();
    let cipher = Aes256SivAead::new(key);

    let try_decrypt = cipher.decrypt(nonce, url.as_bytes()).map_err(|_| "Incorrect password supplied".to_string())?;
    let url = String::from_utf8(try_decrypt).map_err(|_| "Decrypted URL is not valid UTF-8".to_string())?;

    let (ws_stream, _) = connect_async(url.replace("http", "ws")) //TODO: Change to https
        .await
        .expect("Couldn't connect to chat");
    let (mut write, read) = ws_stream.split();

    let join_cmd = SendData::Join(
        Join { 
            username: username,
            pub_key: pub_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF).expect("Couldn't serialize public key"), 
        }
    );

    write
        .send(Text(serde_json::to_string(&join_cmd).unwrap()))
        .await
        .expect("Couldn't send join command");

    *CLIENT.lock().await = Client {
        write: Some(write),
        pub_key: Some(pub_key),
        priv_key: Some(priv_key),
        chat_key: None
    };

    tokio::spawn(async move {
        let (tx, rx) = mpsc::unbounded_channel::<RecvData>();
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<bool>(); //Can't use oneshot channel due it consuming itself during a send

        let msg_handle = window.listen("host-message", move |e| {
            if e.payload().is_none() {
                return;
            }

            println!("{:#?}", e.payload());
            
            tokio::spawn(async move {
                let mut client = CLIENT.lock().await;

                let try_key = client.chat_key.as_mut();
                if try_key.is_none() {
                    //TODO: Display user error message
                }
                let key: &Key<Aes256SivAead> = try_key.unwrap().as_slice().into();
                let cipher = Aes256SivAead::new(key);
                let mut nonce: [u8; 16] = [0; 16];
                OsRng.fill_bytes(&mut nonce);
                let nonce = Nonce::from_slice(&nonce);
                let cipher_text = cipher.encrypt(&nonce, e.payload().unwrap().as_bytes()).expect("Couldn't encrypt user message");

                let send_data = serde_json::to_string(&SendData::EncData(EncData {
                    nonce: nonce.to_vec(),
                    data: cipher_text
                })).expect("Couldn't convert send data to string");

                client
                    .write
                    .as_mut()
                    .unwrap()
                    .send(Text(send_data))
                    .await
                    .expect("Couldn't send host message");
            });
        });

        let exit_handle = window.listen("client_exit", move |_| {
            tokio::spawn(async move {
                let mut client = CLIENT.lock().await;
                client
                    .write
                    .as_mut()
                    .unwrap()
                    .send(Text(serde_json::to_string(&SendData::Exit).unwrap()))
                    .await
                    .expect("Couldn't send host message");
                let _ = client.write.as_mut().unwrap().close().await;
                client.write = None;
            });
            let _ = shutdown_tx.send(true);
        });

        let read_handle = tokio::spawn(async move {
            read.for_each(|message_res| async {
                if let Ok(message) = message_res {
                    println!("{}", message);
                    let parsed = serde_json::from_str::<RecvData>(&message.to_string());
                    if let Ok(recv_data) = parsed {
                        tx.send(recv_data).expect("Couldn't send recieve data");
                    }
                }
            })
            .await;
        });

        tokio::select! {
            _ = shutdown_rx.recv() => {
                read_handle.abort();
                window.unlisten(exit_handle);
                window.unlisten(msg_handle);
                shutdown_rx.close();
            },
            _ = handle_recv_data(rx, window.clone()) => {}
        }
    });
    Ok(())
}
