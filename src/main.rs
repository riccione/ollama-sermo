use axum::{
    routing::{get, post},
    Router,
    serve,
};
use tokio::net::TcpListener;
mod include;
mod chat;

#[tokio::main]
async fn main() {
    let app = Router::new()
        //.route("/", get(include::index))
        .route("/", get(include::index))
        .route("/chat", post(chat::chat))
        .route("/models", get(chat::models))
        .route("/static/{file}", get(include::static_file)); // for css

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running at http://{addr}");
    serve(listener, app).await.unwrap();
}
