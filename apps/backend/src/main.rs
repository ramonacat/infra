#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::{sync::Arc, time::Duration};

use ::tracing::{Level, Span};
use axum::{
    body::{Body, BoxBody},
    extract::MatchedPath,
    http::{uri::Scheme, Request, Response},
    routing::{get, post},
    Router,
};
use rand::thread_rng;
use service_accounts::ServiceAccountRepository;
use tracing::setup_tracing_subscriber;

use crate::service_accounts::initialize_root_account;

mod database;
mod secrets;
mod service_accounts;
mod tracing;

mod blog;

fn make_span(request: &Request<Body>) -> Span {
    let request_uri = request.uri().to_string();
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| &request_uri, MatchedPath::as_str);
    let content_length: u64 = request
        .headers()
        .get("Content-Length")
        .map_or(0, |x| x.to_str().unwrap_or("0").parse().unwrap_or(0));
    let user_agent = request
        .headers()
        .get("User-Agent")
        .map_or("", |x| x.to_str().unwrap_or(""));

    ::tracing::span!(
        Level::INFO,
        "",
        otel.name = format!("{} {}", request.method().as_str(), matched_path),
        network.protocol.name = request.uri().scheme().map_or("http", Scheme::as_str),
        http.request.body.size = content_length,
        http.request.method = request.method().as_str(),
        user_agent.original = user_agent
    )
}

const fn on_request(_request: &Request<Body>, _span: &Span) {}

fn on_response(response: &Response<BoxBody>, _latency: Duration, span: &Span) {
    let content_length: u64 = response
        .headers()
        .get("Content-Length")
        .map_or(0, |x| x.to_str().unwrap_or("0").parse().unwrap_or(0));
    span.record("http.response.status_code", response.status().as_u16());
    span.record("http.response.body.size", content_length);
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        dotenvy::dotenv().expect("Failed to load .env");
    }

    setup_tracing_subscriber().expect("Failed to setup tracing!");

    let db_pool = database::connect(database::AccessLevel::App)
        .await
        .expect("Database connection failed");
    let db_pool = Arc::new(db_pool);

    let service_account_repository = Arc::new(ServiceAccountRepository::new(db_pool.clone()));
    initialize_root_account(service_account_repository.clone(), thread_rng)
        .await
        .expect("Failed to init root account");

    let asset_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "dist".to_string());

    let assets_service = tower_http::services::ServeDir::new(asset_path);

    let blog_repository = Arc::new(blog::posts::Repository::new(db_pool.clone()));
    let blog = blog::Blog::new(db_pool.clone());
    let blog = Arc::new(blog);

    let api = Router::new()
        .route("/posts", post(blog::route_api_post_posts))
        .layer(axum::middleware::from_fn_with_state(
            service_account_repository,
            service_accounts::middleware,
        ))
        .with_state(blog_repository.clone());

    let application = Router::new()
        .route("/", get(blog::route_main))
        .route("/posts/:id", get(blog::route_posts_id))
        .with_state(blog)
        .nest("/api", api)
        .fallback_service(assets_service)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(make_span)
                .on_request(on_request)
                .on_response(on_response),
        );

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(application.into_make_service())
        .await
        .unwrap();
}
