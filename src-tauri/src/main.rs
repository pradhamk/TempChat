mod client;
mod server;
mod structs;

use self::client::client::join_chat;
use self::server::chat::create_chat;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_chat, join_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
