use crate::ports::pty::{PtyHandle, PtyReader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

pub type SessionMap = Arc<Mutex<HashMap<String, PtyHandle>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
}

#[derive(Clone, Serialize)]
struct PtyDataPayload {
    session_id: String,
    data: Vec<u8>,
}

#[derive(Clone, Serialize)]
struct PtyExitPayload {
    session_id: String,
    code: i32,
}

#[derive(Clone, Serialize)]
struct PtyErrorPayload {
    session_id: String,
    error_type: String,
    message: String,
}

#[tauri::command]
pub async fn create_terminal_session(
    app: AppHandle,
    sessions: State<'_, SessionMap>,
    shell: Option<String>,
    cwd: Option<String>,
) -> Result<CreateSessionResponse, String> {
    let session_id = Uuid::new_v4().to_string();
    let shell_path = shell.unwrap_or_else(|| {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
    });
    let cwd_ref = cwd.as_deref();

    let (handle, reader) = PtyHandle::spawn(&shell_path, cwd_ref, 24, 80)?;

    {
        let mut map = sessions.lock().map_err(|e| e.to_string())?;
        map.insert(session_id.clone(), handle);
    }

    // Spawn read loop in background
    let sid = session_id.clone();
    let app_handle = app.clone();
    std::thread::spawn(move || {
        pty_read_loop(reader, sid, app_handle);
    });

    Ok(CreateSessionResponse { session_id })
}

fn pty_read_loop(mut reader: PtyReader, session_id: String, app: AppHandle) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read_chunk(&mut buf) {
            Ok(0) => {
                // EOF - shell exited
                let _ = app.emit(
                    "pty:exit",
                    PtyExitPayload {
                        session_id: session_id.clone(),
                        code: 0,
                    },
                );
                break;
            }
            Ok(n) => {
                let _ = app.emit(
                    "pty:data",
                    PtyDataPayload {
                        session_id: session_id.clone(),
                        data: buf[..n].to_vec(),
                    },
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "pty:error",
                    PtyErrorPayload {
                        session_id: session_id.clone(),
                        error_type: "read_io_error".to_string(),
                        message: e.to_string(),
                    },
                );
                break;
            }
        }
    }
}

#[tauri::command]
pub async fn write_to_pty(
    sessions: State<'_, SessionMap>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let map = sessions.lock().map_err(|e| e.to_string())?;
    let handle = map
        .get(&session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))?;
    handle.write(&data)
}

#[tauri::command]
pub async fn resize_pty(
    sessions: State<'_, SessionMap>,
    session_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let map = sessions.lock().map_err(|e| e.to_string())?;
    let handle = map
        .get(&session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))?;
    handle.resize(rows, cols)
}

#[tauri::command]
pub async fn close_terminal_session(
    sessions: State<'_, SessionMap>,
    session_id: String,
) -> Result<(), String> {
    let mut map = sessions.lock().map_err(|e| e.to_string())?;
    if let Some(handle) = map.remove(&session_id) {
        let _ = handle.kill();
    }
    Ok(())
}
