#[path = "../database.rs"]
mod database;
#[path = "../secrets.rs"]
mod secrets;

#[tokio::main]
async fn main() {
    let db_pool = crate::database::connect(database::DatabaseAccessLevel::Superuser)
        .await
        .expect("Database connection failed");

    sqlx::migrate!("./migrations/")
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations!");
}
