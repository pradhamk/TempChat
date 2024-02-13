#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::SocketAddr, sync::Arc, collections::HashMap};
use rand::Rng;
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast};
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt, lock::Mutex};
use localtunnel_client::{open_tunnel, ClientConfig};
use nanoid::nanoid;
use tauri::{command, Window};
use once_cell::sync::Lazy;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message::Text};

static PEER_MAP: Lazy<Mutex<HashMap<String, SplitSink<WebSocketStream<tokio::net::TcpStream>, tokio_tungstenite::tungstenite::Message>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Message {
    command: String,
    sender_id: String,
    content: String,
}

#[command]
async fn create_chat(username: String, user_limit: u8, window: Window) -> Result<String, String> {
    let port = rand::thread_rng().gen_range(10_000..=20_000);
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|_| format!("Unable to bind to port {}", port))?;
    println!("Started listening on port {}", port);

    let (notify_shutdown, _) = broadcast::channel(1);
    let alphabet: [char; 36] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    let chat_id = nanoid!(16, &alphabet);
    let config = ClientConfig {
        server: Some("https://loca.lt".into()),
        subdomain: Some(chat_id),
        local_host: Some("127.0.0.1".into()),
        local_port: port,
        shutdown_signal: notify_shutdown.clone(),
        max_conn: user_limit,
        credential: None,
    };

    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(handle_connection(stream, window.clone()));
            }
        }
    });

    Ok(format!("http://127.0.0.1:{}", port)) // For testing purposes
}

async fn handle_connection(stream: TcpStream, window: Window) {
    println!("New client connected: {:#?}", stream.peer_addr());
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
        let (mut write, mut read) = ws_stream.split();
        if let Some(Ok(msg)) = read.next().await {
            let message: Result<Message, _> = serde_json::from_str(&msg.to_string());
            if let Ok(message_data) = message {
                if message_data.command != "join" {
                    write.close().await.expect("Couldn't close connection");
                    return;
                }

                PEER_MAP.lock().await.insert(message_data.sender_id, write);
                
                while let Some(Ok(content)) = read.next().await {
                    println!("{}", content);
                    if content.is_text() && !content.is_empty() {
                        let try_msg: Result<Message, serde_json::Error> = serde_json::from_str(&content.to_string());
                        if try_msg.is_err() {
                            continue;
                        }
                        let data = try_msg.unwrap();
                        if data.command != "chat" {
                            continue;
                        }
                        let string_data = serde_json::to_string(&data).expect("Couldn't convert message to string");
                        
                        for write in PEER_MAP.lock().await.values_mut() {
                            write.send(Text(string_data.clone())).await.expect("Couldn't relay message to clients");
                        }
                        
                        window.emit("new-message", string_data).expect("Couldn't emit message");
                    }
                }
            } else {
                write.close().await.expect("Couldn't close connection");
            }
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}