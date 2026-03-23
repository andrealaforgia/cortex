mod commands;
mod domain;
mod ports;

use commands::ai::ApiKeyState;
use commands::config::StorageState;
use commands::terminal::SessionMap;
use ports::storage::Storage;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize storage
    let app_data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ai-terminal");
    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
    let db_path = app_data_dir.join("data.db");
    let storage = Arc::new(
        Storage::new(&db_path).expect("Failed to initialize database"),
    );

    // Load API key from environment (MVP: env var, future: keychain)
    let api_key = std::env::var("ANTHROPIC_API_KEY").ok();

    tauri::Builder::default()
        .manage(SessionMap::new(Mutex::new(HashMap::new())))
        .manage(ApiKeyState::new(Mutex::new(api_key)))
        .manage(StorageState::clone(&storage))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::terminal::create_terminal_session,
            commands::terminal::write_to_pty,
            commands::terminal::resize_pty,
            commands::terminal::close_terminal_session,
            commands::ai::ai_translate_command,
            commands::ai::ai_explain_error,
            commands::ai::ai_chat,
            commands::ai::ai_cancel,
            commands::config::get_config,
            commands::config::set_config,
            commands::config::store_api_key,
            commands::config::has_api_key,
            commands::config::delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
