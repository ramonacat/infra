use std::sync::Arc;

use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub struct Read {
    db_pool: Arc<Pool<Postgres>>,
}

pub struct LatestPost {
    pub id: Uuid,
    pub title: String,
}

pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
}

impl Read {
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        Self { db_pool }
    }

    pub async fn single(&self, id: Uuid) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as!(
            Post,
            "SELECT id, title, content FROM posts WHERE id = $1",
            id
        )
        .fetch_optional(self.db_pool.as_ref())
        .await
    }

    pub async fn latest(&self, count: i64) -> Result<Vec<LatestPost>, sqlx::Error> {
        sqlx::query_as!(
            LatestPost,
            "SELECT id, title FROM posts ORDER BY date_published DESC LIMIT $1",
            count
        )
        .fetch_all(self.db_pool.as_ref())
        .await
    }
}
