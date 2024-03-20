use std::collections::HashMap;

use crate::server::proto::{ChatData, RecvData};
use crate::server::socket::handle::{
    chat_shutdown, handle_connection, handle_user_message, CHAT_DATA,
};
use crate::server::socket::handle::{close_client, handle_message};
use crate::structs::UserMessage;
use crate::utils;
use aes_siv::{
    aead::{KeyInit, OsRng},
    Aes256SivAead,
};
use futures_util::StreamExt;
use localtunnel_client::{open_tunnel, ClientConfig};
use nanoid::nanoid;
use rand::Rng;
use tauri::{command, Window};
use tokio::sync::mpsc;
use tokio::{net::TcpListener, sync::broadcast};

async fn handle_channel_message(
    mut rx: mpsc::UnboundedReceiver<(RecvData, String)>,
    window: Window,
) {
    loop {
        let data = rx.recv().await;
        if let Some((message, uid)) = data {
            if let Err(err) = handle_message(&message, &window, &uid).await {
                close_client(&uid).await;
                return;
            }
        }
    }
}

#[command]
pub async fn create_chat(
    username: String,
    user_limit: i32,
    password: String,
    window: Window,
) -> Result<String, String> {
    let port = rand::thread_rng().gen_range(10_000..=20_000);
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|_| format!("Unable to bind to port {}", port))?;

    let key = Aes256SivAead::generate_key(&mut OsRng);
    let cipher = Aes256SivAead::new(&key);

    *CHAT_DATA.lock().await = ChatData {
        key_cipher: cipher,
        key: key.to_vec(),
        user_limit: user_limit,
        host_username: username,
        peer_map: HashMap::new(),
    };

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
        max_conn: user_limit.clone() as u8 + 5, //Allow more socket connections
        credential: None,
    };

    let tunnel_url = open_tunnel(config).await.expect("Couldn't open tunnel");

    let join_url_res = utils::create_join_url(tunnel_url, password).await;
    tokio::spawn(async move {
        let (tx, rx) = mpsc::unbounded_channel::<(RecvData, String)>();
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<bool>();

        let shutdown_handler = window.listen("shutdown", move |_| {
            shutdown_tx
                .send(true)
                .expect("Couldn't send shutdown channel msg");
        });

        let window_clone = window.clone();
        let host_handle = window.listen("host-message", move |e| {
            if let Some(payload) = e.payload() {
                if let Ok(message) = serde_json::from_str::<UserMessage>(&payload) {
                    let window_clone = window_clone.clone();
                    tokio::spawn(async move {
                        if let Err(err) = handle_user_message(&message, None, &window_clone).await {
                            println!("Couldn't send host message: {:?}", err);
                        }
                    });
                } else {
                    println!("Host message conversion error");
                }
            }
        });

        let conn_handle = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Ok(Some((mut read, uid))) = handle_connection(stream).await {
                        while let Some(Ok(content)) = read.next().await {
                            if let Ok(message) =
                                serde_json::from_str::<RecvData>(&content.to_string())
                            {
                                tx.send((message, uid.clone()))
                                    .expect("Couldn't send message over channel");
                            }
                        }
                    } else {
                        println!("Client connection error");
                    }
                });
            }
        });

        tokio::select! {
            _ = handle_channel_message(rx, window.clone()) => {}
            _ = shutdown_rx.recv() => {
                chat_shutdown().await;
                let _ = notify_shutdown.send(());
                conn_handle.abort();
                window.unlisten(shutdown_handler);
                window.unlisten(host_handle);
                shutdown_rx.close();
            }
        }
    });

    join_url_res
}
