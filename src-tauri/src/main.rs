mod server;
mod client;
mod structs;

use self::server::chat::create_chat;
use self::client::client::join_chat;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_chat, join_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
