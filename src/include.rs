use axum::{
    extract::{Path},
    response::{Html, IntoResponse, Response},
    http::{header, HeaderMap, HeaderValue, StatusCode},
};

const INDEX_HTML: &str = include_str!("../static/index.html");
const STYLE_CSS: &str = include_str!("../static/simple.min.css");

pub async fn index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn static_file(Path(file): Path<String>) -> Response {
    match file.as_str() {
        "index.html" => serve(INDEX_HTML, "text/html").into_response(),
        "simple.min.css" => serve(STYLE_CSS, "text/css").into_response(),
        _ => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

fn serve(content: &'static str, content_type: &'static str) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    (headers, content).into_response()
}
