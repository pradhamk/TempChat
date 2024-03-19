use std::sync::Arc;

use crate::{
    client::proto::{Client, SendData},
    structs::{BroadcastMessage, Join}, utils,
};
use aes_siv::{Aes256SivAead, Key, KeyInit};
use futures_util::{lock::Mutex, SinkExt, StreamExt};
use once_cell::sync::Lazy;
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
                        let decrypted_res = utils::decrypt_message(&enc_data, &Aes256SivAead::new(key)).await;
                        match decrypted_res {
                            Ok(dec_data) => {
                                if let Ok(broadcast_data) = serde_json::from_str::<BroadcastMessage>(&String::from_utf8(dec_data).expect("Value isn't string")) {
                                    println!("Broadcast Data: {:#?}", broadcast_data);
                                    window
                                    .emit(
                                        "new-message",
                                        serde_json::to_string(&broadcast_data).unwrap(),
                                    )
                                    .unwrap();
                                }
                            },
                            Err(err) => {
                                send_err(&window, err).await;
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
                    match dec_data {
                        Ok(chat_key) => {
                            client.chat_key = Some(chat_key);
                        },
                        Err(e) => {
                            send_err(&window, e.to_string()).await;
                        }
                    }
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

pub async fn client_exit() {
    let mut client = CLIENT.lock().await;
    let _ = client
        .write
        .as_mut()
        .unwrap()
        .send(Text(serde_json::to_string(&SendData::Exit).unwrap()))
        .await;
    let _ = client.write.as_mut().unwrap().close().await;
    client.write = None;
}

async fn send_err(window: &Window, err: String) {
    window.emit("error", err).expect("Couldn't emit error");
}

#[command]
pub async fn join_chat(username: String, chat_url: String, password: String, window: Window) -> Result<(), String> {
    let mut rng = rand::rngs::OsRng::default();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Couldn't generate user private key");
    let pub_key = RsaPublicKey::from(&priv_key);

    let url = utils::parse_join_url(chat_url, password).await?;

    let (ws_stream, _) = connect_async(url.replace("https", "wss").replace("http", "ws"))
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

        let window_clone = window.clone();
        let msg_handle = window.listen("host-message", move |e| {
            if e.payload().is_none() {
                return;
            }

            let error_window = window_clone.clone();
            tokio::spawn(async move {
                let mut client = CLIENT.lock().await;

                let try_key = client.chat_key.as_mut();
                if try_key.is_none() {
                    send_err(&error_window, "Server chat key not found".into()).await;
                }
                let key: &Key<Aes256SivAead> = try_key.unwrap().as_slice().into();
                let cipher = Aes256SivAead::new(key);
                let encrypted = utils::encrypt_message(e.payload().unwrap().into(), &cipher).await;
                if encrypted.is_err() {
                    send_err(&error_window, "Couldn't encrypt user message".into()).await;
                }
                let send_data = serde_json::to_string(&SendData::EncData(encrypted.unwrap())).expect("Couldn't convert send data to string");

                let send_res = client
                    .write
                    .as_mut()
                    .unwrap()
                    .send(Text(send_data))
                    .await;
                if send_res.is_err() {
                    let _ = error_window.emit("error", "Connection closed");
                }
            });
        });

        let exit_handle = window.listen("client_exit", move |_| {
            tokio::spawn(async move {
                client_exit().await;
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
