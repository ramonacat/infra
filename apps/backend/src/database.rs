use sqlx::postgres::PgPoolOptions;

use crate::secrets;

pub enum AccessLevel {
    #[allow(unused)]
    Superuser,
    #[allow(unused)]
    App,
}

pub async fn connect(access_level: AccessLevel) -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
    let secret_name = match access_level {
        AccessLevel::Superuser => "db-backend-superuser",
        AccessLevel::App => "db-backend-app",
    };

    let username = secrets::read(secret_name, "username").expect("Failed to read username");
    let password = secrets::read(secret_name, "password").expect("Failed to read password");

    PgPoolOptions::new()
        .max_connections(50) // TODO: tune
        .connect(&format!(
            "postgres://{username}:{password}@db-backend-rw/app"
        ))
        .await
}
