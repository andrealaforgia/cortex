use crate::commands::ai::ApiKeyState;
use crate::ports::claude_api;
use crate::ports::storage::Storage;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

pub type StorageState = Arc<Storage>;

#[tauri::command]
pub async fn get_config(
    storage: State<'_, StorageState>,
) -> Result<HashMap<String, String>, String> {
    let pairs = storage.get_all_config()?;
    Ok(pairs.into_iter().collect())
}

#[tauri::command]
pub async fn set_config(
    storage: State<'_, StorageState>,
    config: HashMap<String, String>,
) -> Result<(), String> {
    for (key, value) in config {
        storage.set_config(&key, &value)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn store_api_key(
    api_key_state: State<'_, ApiKeyState>,
    key: String,
) -> Result<serde_json::Value, String> {
    // Validate the key
    match claude_api::validate_api_key(&key).await {
        Ok(true) => {
            let mut state = api_key_state.lock().map_err(|e| e.to_string())?;
            *state = Some(key);
            Ok(serde_json::json!({ "valid": true }))
        }
        Ok(false) => Ok(serde_json::json!({
            "valid": false,
            "error": "Invalid API key"
        })),
        Err(e) => Ok(serde_json::json!({
            "valid": false,
            "error": e
        })),
    }
}

#[tauri::command]
pub async fn has_api_key(
    api_key_state: State<'_, ApiKeyState>,
) -> Result<serde_json::Value, String> {
    let state = api_key_state.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "exists": state.is_some() }))
}

#[tauri::command]
pub async fn delete_api_key(
    api_key_state: State<'_, ApiKeyState>,
) -> Result<(), String> {
    let mut state = api_key_state.lock().map_err(|e| e.to_string())?;
    *state = None;
    Ok(())
}
