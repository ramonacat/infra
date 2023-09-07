#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::{collections::HashMap, sync::Arc};

use axum::{routing::get, Router};
use opentelemetry::{runtime, sdk::Resource, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use rand::{thread_rng, CryptoRng, Rng};
use service_accounts::{ServiceAccount, ServiceAccountRepository, ServiceAccountToken};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::Level;
use tracing_subscriber::{filter::LevelFilter, prelude::*};
use uuid::Uuid;

mod database;
mod secrets;
mod service_accounts;

const ROOT_ACCOUNT_NAME: &str = "root";

async fn initialize_root_account<TCryptoRng: CryptoRng + Rng>(
    repository: Arc<ServiceAccountRepository>,
    csprng: impl (FnOnce() -> TCryptoRng) + Send,
) -> Result<(), sqlx::Error> {
    let current_account = repository.find_by_name(ROOT_ACCOUNT_NAME).await?;

    if current_account.is_none() {
        let mut account = ServiceAccount::create(Uuid::new_v4(), ROOT_ACCOUNT_NAME.into());

        account.add_token(ServiceAccountToken::create(Uuid::new_v4(), csprng));

        repository.save(account).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut metadata = HashMap::<String, String>::with_capacity(1);
    metadata.insert(
        "x-honeycomb-team".to_string(),
        secrets::read("honeycomb-key", "HONEYCOMB_KEY")
            .expect("Failed to read the honeycomb secret"),
    );

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_http_client(reqwest::Client::default())
                .with_endpoint("https://api.honeycomb.io/v1/traces")
                .with_headers(metadata),
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "backend",
            )])),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to create the opentelemetry tracer");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::Layer::default().with_writer(std::io::stdout))
        .with(telemetry.with_filter(LevelFilter::INFO))
        .init();

    tracing::info!("Initialized tracing!");

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
        .route(
            "/api",
            get(|| async {
                tracing::info_span!("request");

                tracing::info!("Received an HTTP request");
                "Hello World!"
            }),
        )
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
