use crate::domain::context::{self, ShellContext};
use crate::domain::redaction::RedactionEngine;
use crate::ports::claude_api::{self, ChatMessage, StreamEvent};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

pub type ApiKeyState = Arc<Mutex<Option<String>>>;

#[derive(Clone, Serialize)]
struct AiStreamChunkPayload {
    request_id: String,
    text: String,
}

#[derive(Clone, Serialize)]
struct AiStreamEndPayload {
    request_id: String,
    model: String,
    input_tokens: u32,
    output_tokens: u32,
    stop_reason: String,
}

#[derive(Clone, Serialize)]
struct AiErrorPayload {
    request_id: String,
    error_type: String,
    message: String,
}

fn get_api_key(api_key_state: &ApiKeyState) -> Result<String, String> {
    let key = api_key_state.lock().map_err(|e| e.to_string())?;
    key.clone().ok_or_else(|| "API key not configured".to_string())
}

#[tauri::command]
pub async fn ai_translate_command(
    app: AppHandle,
    api_key_state: State<'_, ApiKeyState>,
    query: String,
    shell_context: ShellContext,
) -> Result<serde_json::Value, String> {
    let request_id = Uuid::new_v4().to_string();
    let api_key = get_api_key(&api_key_state)?;

    let redaction = RedactionEngine::new();
    let system_prompt = context::build_system_prompt(&shell_context, &redaction);

    let rid = request_id.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: format!("Translate to a shell command: {}", query),
        }];

        let rid_clone = rid.clone();
        let app_clone = app_handle.clone();

        let result = claude_api::send_message_streaming(
            &api_key,
            "claude-sonnet-4-20250514",
            &system_prompt,
            &messages,
            1024,
            move |event| match event {
                StreamEvent::TextDelta(text) => {
                    let _ = app_clone.emit(
                        "ai:stream-chunk",
                        AiStreamChunkPayload {
                            request_id: rid_clone.clone(),
                            text,
                        },
                    );
                }
                StreamEvent::Completion(meta) => {
                    let _ = app_clone.emit(
                        "ai:stream-end",
                        AiStreamEndPayload {
                            request_id: rid_clone.clone(),
                            model: meta.model,
                            input_tokens: meta.input_tokens,
                            output_tokens: meta.output_tokens,
                            stop_reason: meta.stop_reason,
                        },
                    );
                }
                StreamEvent::Error(msg) => {
                    let _ = app_clone.emit(
                        "ai:error",
                        AiErrorPayload {
                            request_id: rid_clone.clone(),
                            error_type: "api_error".to_string(),
                            message: msg,
                        },
                    );
                }
            },
        )
        .await;

        if let Err(e) = result {
            let _ = app_handle.emit(
                "ai:error",
                AiErrorPayload {
                    request_id: rid.clone(),
                    error_type: "request_failed".to_string(),
                    message: e,
                },
            );
        }
    });

    Ok(serde_json::json!({ "request_id": request_id }))
}

#[tauri::command]
pub async fn ai_explain_error(
    app: AppHandle,
    api_key_state: State<'_, ApiKeyState>,
    command: String,
    output: String,
    exit_code: i32,
    shell_context: ShellContext,
) -> Result<serde_json::Value, String> {
    let request_id = Uuid::new_v4().to_string();
    let api_key = get_api_key(&api_key_state)?;

    let redaction = RedactionEngine::new();
    let prompt = context::build_error_diagnosis_prompt(
        &command,
        &output,
        exit_code,
        &shell_context,
        &redaction,
    );

    let rid = request_id.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let rid_clone = rid.clone();
        let app_clone = app_handle.clone();

        let result = claude_api::send_message_streaming(
            &api_key,
            "claude-sonnet-4-20250514",
            "You are a terminal error diagnosis assistant. Be concise and actionable.",
            &messages,
            1024,
            move |event| match event {
                StreamEvent::TextDelta(text) => {
                    let _ = app_clone.emit(
                        "ai:stream-chunk",
                        AiStreamChunkPayload {
                            request_id: rid_clone.clone(),
                            text,
                        },
                    );
                }
                StreamEvent::Completion(meta) => {
                    let _ = app_clone.emit(
                        "ai:stream-end",
                        AiStreamEndPayload {
                            request_id: rid_clone.clone(),
                            model: meta.model,
                            input_tokens: meta.input_tokens,
                            output_tokens: meta.output_tokens,
                            stop_reason: meta.stop_reason,
                        },
                    );
                }
                StreamEvent::Error(msg) => {
                    let _ = app_clone.emit(
                        "ai:error",
                        AiErrorPayload {
                            request_id: rid_clone.clone(),
                            error_type: "api_error".to_string(),
                            message: msg,
                        },
                    );
                }
            },
        )
        .await;

        if let Err(e) = result {
            let _ = app_handle.emit(
                "ai:error",
                AiErrorPayload {
                    request_id: rid,
                    error_type: "request_failed".to_string(),
                    message: e,
                },
            );
        }
    });

    Ok(serde_json::json!({ "request_id": request_id }))
}

#[tauri::command]
pub async fn ai_chat(
    app: AppHandle,
    api_key_state: State<'_, ApiKeyState>,
    message: String,
    conversation_id: Option<String>,
    shell_context: ShellContext,
) -> Result<serde_json::Value, String> {
    let request_id = Uuid::new_v4().to_string();
    let conv_id = conversation_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let api_key = get_api_key(&api_key_state)?;

    let redaction = RedactionEngine::new();
    let system_prompt = context::build_system_prompt(&shell_context, &redaction);

    let rid = request_id.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: message,
        }];

        let rid_clone = rid.clone();
        let app_clone = app_handle.clone();

        let result = claude_api::send_message_streaming(
            &api_key,
            "claude-sonnet-4-20250514",
            &system_prompt,
            &messages,
            2048,
            move |event| match event {
                StreamEvent::TextDelta(text) => {
                    let _ = app_clone.emit(
                        "ai:stream-chunk",
                        AiStreamChunkPayload {
                            request_id: rid_clone.clone(),
                            text,
                        },
                    );
                }
                StreamEvent::Completion(meta) => {
                    let _ = app_clone.emit(
                        "ai:stream-end",
                        AiStreamEndPayload {
                            request_id: rid_clone.clone(),
                            model: meta.model,
                            input_tokens: meta.input_tokens,
                            output_tokens: meta.output_tokens,
                            stop_reason: meta.stop_reason,
                        },
                    );
                }
                StreamEvent::Error(msg) => {
                    let _ = app_clone.emit(
                        "ai:error",
                        AiErrorPayload {
                            request_id: rid_clone.clone(),
                            error_type: "api_error".to_string(),
                            message: msg,
                        },
                    );
                }
            },
        )
        .await;

        if let Err(e) = result {
            let _ = app_handle.emit(
                "ai:error",
                AiErrorPayload {
                    request_id: rid,
                    error_type: "request_failed".to_string(),
                    message: e,
                },
            );
        }
    });

    Ok(serde_json::json!({
        "request_id": request_id,
        "conversation_id": conv_id,
    }))
}

#[tauri::command]
pub async fn ai_cancel(_request_id: String) -> Result<(), String> {
    // MVP: cancel is a no-op. Future: track active requests and abort them.
    Ok(())
}
