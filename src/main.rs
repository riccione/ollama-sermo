use axum::{
    extract::{Query, Path},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
    serve,
    body::Body,
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use tokio::net::TcpListener;
use serde_json::Value;
use tokio_stream::StreamExt;
use bytes::Bytes;

static STATIC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

#[derive(Deserialize)]
struct ChatRequest {
    host: String,
    model: String,
    prompt: String,
    system_prompt: String,
}

#[derive(Deserialize)]
struct ModelsQuery {
    host: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/chat", post(chat))
        .route("/models", get(models))
        .route("/{file}", get(static_file)); // for css and js

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running at http://{addr}");

    serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    let file = STATIC_DIR.get_file("index.html").unwrap();
    Html(file.contents_utf8().unwrap().to_string())
}

async fn static_file(Path(file): Path<String>) -> impl IntoResponse {
    match STATIC_DIR.get_file(&file) {
        Some(file) => {
            let contents = file.contents();
            let content_type = match file.path().extension().and_then(|ext| ext.to_str()) {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("ico") => "image/x-icon",
                Some("json") => "application/json",
                _ => "application/octet-stream", // fallback
            };

            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type),
            );

            (headers, contents.to_vec()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

async fn chat(Json(payload): Json<ChatRequest>) -> impl IntoResponse {
    let url = format!("{}/api/chat", payload.host);
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": payload.model,
        "stream": true,
        "messages": [
            { "role": "system", "content": payload.system_prompt },
            { "role": "user", "content": payload.prompt }
        ]
    });

    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to connect to Ollama".to_string(),
            )
                .into_response();
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

async fn models(Query(params): Query<ModelsQuery>) -> impl IntoResponse {
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
