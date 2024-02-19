use std::sync::Arc;
use rand::Rng;
use tokio::{net::TcpListener, sync::broadcast};
use localtunnel_client::{open_tunnel, ClientConfig};
use nanoid::nanoid;
use tauri::{command, Window};
use futures_util::lock::Mutex;
use crate::server::socket::handle::{chat_shutdown, handle_connection, handle_user_message, USERNAME, USER_LIMIT};
use crate::server::proto::RecvData;

#[command]
pub async fn create_chat(username: String, user_limit: i32, window: Window) -> Result<String, String> {
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
        max_conn: user_limit.clone() as u8 + 5, //Allow more socket connections
        credential: None,
    };

    *USERNAME.lock().await = username;
    *USER_LIMIT.lock().await = user_limit;

    /*
    let tunnel_url = open_tunnel(config)
        .await
        .expect("Couldn't open tunnel");
    */

    tokio::spawn(async move {
        let shutdown_flag = Arc::new(Mutex::new(false));
        let shutdown_clone = Arc::clone(&shutdown_flag);
        let window_ref = Arc::new(window);

        let shutdown_notifier = window_ref.listen("shutdown", move |_| {
            let shutdown_flag = Arc::clone(&shutdown_clone);
            let notify_shutdown = notify_shutdown.clone();

            tokio::spawn(async move {
                println!("Shutting down server...");
                chat_shutdown().await;
                *shutdown_flag.lock().await = true;
                let _ = notify_shutdown.send(());
            });
        });

        let ref_clone = window_ref.clone();
        let host_notifier = window_ref.listen("host-message", move |e| {
            let wclone = Arc::clone(&ref_clone);
            tokio::spawn(async move {
                if let Some(payload) = e.payload() {
                    if let Ok(message) = serde_json::from_str::<RecvData>(&payload) {
                        if let RecvData::UserMessage(data) = message {
                            if let Err(err) = handle_user_message(&data, &wclone, None).await {
                                println!("Couldn't send host message: {:?}", err);
                            }
                        }
                    } else {
                        println!("Host message conversion error");
                    }
                }
            });
        });

        let wref_clone = window_ref.clone();
        println!("Spawning listener");

        while let Ok((stream, _)) = listener.accept().await {
            println!("Shutting down socket");
            if *shutdown_flag.lock().await {
                window_ref.unlisten(shutdown_notifier);
                window_ref.unlisten(host_notifier);
                return;
            }

            let wclone = Arc::clone(&wref_clone);
            tokio::spawn(async move {
                println!("New connection");
                let _ = handle_connection(stream, &wclone).await;
            });
        }
    });

    Ok(format!("http://127.0.0.1:{}", port)) // For testing purposes
}