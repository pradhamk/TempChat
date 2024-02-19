mod server;

use self::server::chat::create_chat;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}