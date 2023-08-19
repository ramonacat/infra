use axum::{
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use time::OffsetDateTime;

mod secrets;

#[tokio::main]
async fn main() {
    let username = secrets::read("db-backend-app", "username").expect("Failed to read username");
    let password = secrets::read("db-backend-app", "password").expect("Failed to read password");

    let db_pool = PgPoolOptions::new()
        .max_connections(50) // TODO: tune
        .connect(&format!("postgres://{}:{}@db-backend-rw/app", username, password))
        .await
        .expect("Database connection failed");

    let query_result: (OffsetDateTime,) = sqlx::query_as("SELECT NOW()").fetch_one(&db_pool).await.expect("Database query failed");

    println!("Database time: {}", query_result.0);

    // build our application with a single route
    let app = Router::new()
        // TODO is it possible to set the base path?
        .route("/api", get(|| async { "Hello, World!" }));


    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}