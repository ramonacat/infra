use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
};

use serde::Deserialize;
use sqlx::{Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

use self::{
    posts::{read, Post, Repository},
    views::post::render_view,
};

pub mod posts;
mod views;

struct LatestPostView {
    id: Uuid,
    title: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<LatestPostView>,
}

#[derive(Deserialize)]
pub struct QueryString {
    preview: Option<i32>,
}

pub struct Blog {
    posts: read::Read,
}

#[derive(Deserialize)]
pub struct PostCreateRequest {
    id: Uuid,
    title: String,
    content: String,
}

pub async fn route_api_post_posts(
    State(repository): State<Arc<Repository>>,
    Json(request): Json<PostCreateRequest>,
) -> impl IntoResponse {
    repository
        .create(Post {
            id: request.id,
            date_published: OffsetDateTime::now_utc(),
            title: request.title,
            content: request.content,
        })
        .await
        .unwrap();

    (axum::http::StatusCode::CREATED, "")
}

pub async fn route_main(
    Query(query): Query<QueryString>,
    State(blog): State<Arc<Blog>>,
) -> impl IntoResponse {
    if query.preview.unwrap_or(0) != 1 {
        return (axum::http::StatusCode::NOT_FOUND, Html(String::new()));
    }

    let latest = blog.posts.latest(10).await.unwrap();

    let template = IndexTemplate {
        posts: latest
            .into_iter()
            .map(|x| LatestPostView {
                id: x.id,
                title: x.title,
            })
            .collect(),
    };
    (axum::http::StatusCode::OK, Html(template.render().unwrap()))
}

pub async fn route_posts_id(
    Path(id): Path<Uuid>,
    State(blog): State<Arc<Blog>>,
) -> impl IntoResponse {
    let post = blog.posts.single(id).await.unwrap().unwrap();

    let template = render_view(post);

    (axum::http::StatusCode::OK, Html(template.render().unwrap()))
}

impl Blog {
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        Self {
            posts: read::Read::new(db_pool),
        }
    }
}
