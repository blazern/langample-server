mod app_state;
mod graphql;
mod llm;
mod model;
mod panlex;
mod util;
mod kaikki;
mod tatoeba;

use app_state::AppState;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{response::Html, routing::get, Router};
use clap::Parser;
use graphql::schema::{build_schema, AppSchema};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "graphql-parent-path", required = true)]
    graphql_parent_path: String,
    #[arg(long = "api-key-chatgpt", required = true)]
    api_key_chat_gpt: String,
    #[arg(long = "panlex-sqlite-db-path", required = true)]
    panlex_sqlite_db_path: String,
    #[arg(long = "port", default_value = "8080")]
    port: String,
    #[arg(long = "cors-permissive", default_value_t = false)]
    cors_permissive: bool,
}

async fn graphiql(graphql_parent_path: String) -> Html<String> {
    Html(
        GraphiQLSource::build()
            .endpoint(&format!("{graphql_parent_path}graphql"))
            .finish(),
    )
}

fn init_tracing() {
    let crate_name = env!("CARGO_PKG_NAME");
    let filter = EnvFilter::new(format!(
        "info,tower_http=info,async_graphql=info,{crate_name}=debug"
    ));
    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .json()
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    let args = Args::parse();
    let panlex_sqlite_pool = SqlitePool::connect(&args.panlex_sqlite_db_path)
        .await
        .expect("Can't connect to the PanLex DB");
    let app_state = AppState::new(args.api_key_chat_gpt, panlex_sqlite_pool)
        .expect("Failed to create app state");
    let schema: AppSchema = build_schema(app_state.clone());

    let graphql_parent_path = args.graphql_parent_path.clone();
    let app = Router::new()
        .route("/graphiql", get(|| graphiql(graphql_parent_path)))
        .route_service("/graphql", GraphQL::new(schema.clone()))
        .route("/kaikki", get(kaikki::kaikki_proxy::kaikki_proxy))
        .route("/tatoeba", get(tatoeba::tatoeba_proxy::tatoeba_proxy))
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::INFO)),
        );

    let app = if args.cors_permissive {
        app.layer(CorsLayer::permissive())
    } else {
        app
    };

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
