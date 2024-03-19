mod client;
mod server;
mod structs;
mod utils;

use utils::handle_exit;

use self::client::client::join_chat;
use self::server::chat::create_chat;

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_chat, join_chat, exit_app])
        .on_window_event(|event| {
            match event.event() {
                tauri::WindowEvent::CloseRequested { .. } => {
                    handle_exit(event);
                    exit_app();
                },
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
