use sqlx::postgres::PgPoolOptions;

use crate::secrets;

pub enum DatabaseAccessLevel {
    #[allow(unused)]
    Superuser,
    #[allow(unused)]
    App,
}

pub async fn connect(
    access_level: DatabaseAccessLevel,
) -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
    let secret_name = match access_level {
        DatabaseAccessLevel::Superuser => "db-backend-superuser",
        DatabaseAccessLevel::App => "db-backend-app",
    };

    let username = secrets::read(secret_name, "username").expect("Failed to read username");
    let password = secrets::read(secret_name, "password").expect("Failed to read password");

    PgPoolOptions::new()
        .max_connections(50) // TODO: tune
        .connect(&format!(
            "postgres://{}:{}@db-backend-rw/app",
            username, password
        ))
        .await
}
