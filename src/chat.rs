use axum::{
    extract::{Query},
    response::{IntoResponse, Response},
    Json,
    body::Body,
    http::{StatusCode},
};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use bytes::Bytes;

#[derive(Deserialize, Serialize)]
pub struct ChatMsg {
    role: String, // user vs bot
    content: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    host: String,
    model: String,
    messages: Vec<ChatMsg>, // full history
}

#[derive(Deserialize)]
pub struct ModelsQuery {
    host: String,
}

pub async fn chat(Json(payload): Json<ChatRequest>) -> impl IntoResponse {
    let url = format!("{}/api/chat", payload.host);
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": payload.model,
        "stream": true,
        "messages": payload.messages // pass full history to preserve context
    });

    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to connect to Ollama".to_string(),
            ).into_response();
        }
    };

    let stream = res.bytes_stream();

    let output_stream = stream.map(|chunk_result| {
        match chunk_result {
            Ok(chunk) => {
                let text = String::from_utf8_lossy(&chunk);
                let mut collected = String::new();

                for line in text.lines() {
                    if let Ok(parsed) = serde_json::from_str::<Value>(line) {
                        if let Some(content) = parsed["message"]["content"].as_str() {
                            collected.push_str(content);
                        }
                    }
                }

                Ok::<Bytes, std::io::Error>(Bytes::from(collected))
            }
            Err(_) => Ok(Bytes::from("[stream error]")),
        }
    });
    
    Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from_stream(output_stream))
        .unwrap()
}

pub async fn models(Query(params): Query<ModelsQuery>) -> impl IntoResponse {
    let url = format!("{}/api/tags", params.host);
    let client = reqwest::Client::new();

    let res = client.get(&url).send().await;
    match res {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().await.unwrap_or_default();
            let models: Vec<String> = json["models"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                .collect();

            Json(models)
        }
        Err(_) => Json(vec![]),
    }
}
