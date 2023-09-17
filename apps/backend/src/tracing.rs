use std::collections::HashMap;

use opentelemetry::{
    runtime,
    sdk::{trace::Tracer, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use thiserror::Error;
use tracing::info;
use tracing_subscriber::{
    filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    Layer,
};

use crate::secrets;

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("Failed to read secret: {0}")]
    FailedToReadSecret(String),
    #[error("Failed to setup opentelemetry tracer: {0}")]
    FailedToSetupOpentelemetry(String),
}

fn setup_tracer() -> Result<Tracer, SetupError> {
    let mut metadata = HashMap::<String, String>::with_capacity(1);
    metadata.insert(
        "x-honeycomb-team".to_string(),
        secrets::read("honeycomb-key", "HONEYCOMB_KEY")
            .map_err(|e| SetupError::FailedToReadSecret(e.to_string()))?,
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
        .map_err(|e| SetupError::FailedToSetupOpentelemetry(e.to_string()))?;

    Ok(tracer)
}

pub fn setup_tracing_subscriber() -> Result<(), SetupError> {
    let tracer = setup_tracer()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::Layer::default().with_writer(std::io::stdout))
        .with(telemetry.with_filter(LevelFilter::INFO))
        .init();

    info!("Initialized tracing!");

    Ok(())
}
