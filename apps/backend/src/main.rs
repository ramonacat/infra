use axum::{routing::get, Router};
use time::OffsetDateTime;

mod database;
mod secrets;

#[tokio::main]
async fn main() {
    let db_pool = database::connect(database::DatabaseAccessLevel::App)
        .await
        .expect("Database connection failed");

    let query_result: (OffsetDateTime,) = sqlx::query_as("SELECT NOW()")
        .fetch_one(&db_pool)
        .await
        .expect("Database query failed");

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
