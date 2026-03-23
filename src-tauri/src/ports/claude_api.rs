use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub stop_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug)]
pub enum StreamEvent {
    TextDelta(String),
    Completion(StreamMetadata),
    Error(String),
}

pub async fn send_message_streaming(
    api_key: &str,
    model: &str,
    system: &str,
    messages: &[ChatMessage],
    max_tokens: u32,
    on_event: impl Fn(StreamEvent) + Send + 'static,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "system": system,
        "messages": messages,
        "stream": true,
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        on_event(StreamEvent::Error(format!("API error {}: {}", status, body)));
        return Err(format!("API error {}: {}", status, body));
    }

    // Parse SSE stream
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let mut model_name = String::new();
    let mut input_tokens: u32 = 0;
    let mut output_tokens: u32 = 0;
    let mut stop_reason = String::from("end_turn");

    for line in text.lines() {
        if !line.starts_with("data: ") {
            continue;
        }
        let data = &line[6..];
        if data == "[DONE]" {
            break;
        }

        if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
            match event["type"].as_str() {
                Some("message_start") => {
                    if let Some(m) = event["message"]["model"].as_str() {
                        model_name = m.to_string();
                    }
                    if let Some(t) = event["message"]["usage"]["input_tokens"].as_u64() {
                        input_tokens = t as u32;
                    }
                }
                Some("content_block_delta") => {
                    if let Some(text) = event["delta"]["text"].as_str() {
                        on_event(StreamEvent::TextDelta(text.to_string()));
                    }
                }
                Some("message_delta") => {
                    if let Some(sr) = event["delta"]["stop_reason"].as_str() {
                        stop_reason = sr.to_string();
                    }
                    if let Some(t) = event["usage"]["output_tokens"].as_u64() {
                        output_tokens = t as u32;
                    }
                }
                _ => {}
            }
        }
    }

    on_event(StreamEvent::Completion(StreamMetadata {
        model: model_name,
        input_tokens,
        output_tokens,
        stop_reason,
    }));

    Ok(())
}

pub async fn validate_api_key(api_key: &str) -> Result<bool, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 1,
        "messages": [{"role": "user", "content": "hi"}],
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Validation request failed: {}", e))?;

    Ok(response.status().is_success())
}
