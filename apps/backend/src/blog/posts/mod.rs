use std::sync::Arc;

use sqlx::{Pool, Postgres};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

pub mod read;

pub struct Post {
    pub id: Uuid,
    pub date_published: OffsetDateTime,
    pub title: String,
    pub content: String,
}

pub struct Repository {
    db_pool: Arc<Pool<Postgres>>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database Error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl Repository {
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        Self { db_pool }
    }

    pub async fn create(&self, post: Post) -> Result<(), Error> {
        let mut transaction = self.db_pool.begin().await?;

        sqlx::query!(
            "INSERT INTO posts (id, date_published, title, content) VALUES($1, $2, $3, $4)",
            post.id,
            post.date_published,
            post.title,
            post.content
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
}
