#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod socket;

use rand::Rng;
use tokio::{net::TcpListener, sync::broadcast};
use localtunnel_client::{open_tunnel, ClientConfig};
use nanoid::nanoid;
use tauri::{command, Window};
use socket::handle::handle_connection;

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

    /*
    let url = open_tunnel(config)
        .await
        .map_err(|e| format!("Unable to open up tunnel: {}", e))?;
    println!("Tunnel located at {}", url);
    */

    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(
                    handle_connection(stream, window.clone())
                );
            }
        }
    });


    Ok(format!("http://127.0.0.1:{}", port)) // For testing purposes
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}