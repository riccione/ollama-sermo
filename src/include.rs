use axum::{
    extract::{Path},
    response::{Html, IntoResponse},
    http::{header, HeaderMap, HeaderValue, StatusCode},
};
use include_dir::{include_dir, Dir};

static STATIC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

pub async fn index() -> impl IntoResponse {
    let file = STATIC_DIR.get_file("index.html").unwrap();
    Html(file.contents_utf8().unwrap().to_string())
}

pub async fn static_file(Path(file): Path<String>) -> impl IntoResponse {
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
