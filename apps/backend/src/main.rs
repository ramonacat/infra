#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::sync::Arc;

use ::tracing::Level;
use axum::{routing::get, Router};
use rand::thread_rng;
use service_accounts::ServiceAccountRepository;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::setup_tracing_subscriber;

use crate::service_accounts::initialize_root_account;

mod database;
mod secrets;
mod service_accounts;
mod tracing;

#[tokio::main]
async fn main() {
    setup_tracing_subscriber().expect("Failed to setup tracing");

    let db_pool = database::connect(database::AccessLevel::App)
        .await
        .expect("Database connection failed");
    let db_pool = Arc::new(db_pool);

    initialize_root_account(
        Arc::new(ServiceAccountRepository::new(db_pool.clone())),
        thread_rng,
    )
    .await
    .expect("Failed to init root account");

    // build our application with a single route
    let app = Router::new()
        // TODO is it possible to set the base path?
        .route("/api", get(|| async { "Hello World!" }))
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
