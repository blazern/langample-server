use clap::Parser;

use axum::{
    routing::get,
    Router,
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "api-key-chatgpt", required = true)]
    api_key_chat_gpt: String,
}

#[tokio::main]
async fn main() {
    let _args = Args::parse();
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
