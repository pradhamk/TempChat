use std::sync::Arc;

use crate::{
    client::proto::{Client, SendData},
    structs::Join,
};
use futures_util::{lock::Mutex, SinkExt, StreamExt};
use once_cell::sync::Lazy;
use tauri::{command, Window};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message::Text};

use super::proto::RecvData;

static CLIENT: Lazy<Arc<Mutex<Client>>> = Lazy::new(|| Arc::new(Mutex::new(Client { write: None })));

async fn handle_recv_data(mut rx: mpsc::UnboundedReceiver<RecvData>, window: Window) {
    loop {
        let res = rx.recv().await;
        if let Some(recv_data) = res {
            match recv_data {
                RecvData::BroadcastMessage(data) => {
                    window
                        .emit(
                            "new-message",
                            serde_json::to_string(&RecvData::BroadcastMessage(data)).unwrap(),
                        )
                        .unwrap();
                }
                RecvData::JoinMessage(data) => {
                    window
                        .emit(
                            "join",
                            serde_json::to_string(&RecvData::JoinMessage(data)).unwrap(),
                        )
                        .unwrap();
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
pub async fn join_chat(username: String, chat_url: String, window: Window) {
    let (ws_stream, _) = connect_async(chat_url.replace("http", "ws"))
        .await
        .expect("Couldn't connect to chat");
    let (mut write, read) = ws_stream.split();

    let join_cmd = SendData::Join(Join { username: username });

    write
        .send(Text(serde_json::to_string(&join_cmd).unwrap()))
        .await
        .expect("Couldn't send join command");

    CLIENT.lock().await.write = Some(write);

    tokio::spawn(async move {
        let (tx, rx) = mpsc::unbounded_channel::<RecvData>();
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<bool>(); //Can't use oneshot channel due it consuming itself during a send

        let msg_handle = window.listen("host-message", move |e| {
            if e.payload().is_none() {
                return;
            }
            tokio::spawn(async move {
                let mut client = CLIENT.lock().await;
                client
                    .write
                    .as_mut()
                    .unwrap()
                    .send(Text(e.payload().unwrap().to_string()))
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
}
