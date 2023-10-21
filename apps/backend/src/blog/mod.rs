use askama::Template;
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[derive(Deserialize)]
pub struct QueryString {
    preview: Option<i32>,
}

#[allow(clippy::unused_async)]
pub async fn route_main(Query(query): Query<QueryString>) -> impl IntoResponse {
    if query.preview.unwrap_or(0) != 1 {
        return (axum::http::StatusCode::NOT_FOUND, Html(String::new()));
    }

    let template = IndexTemplate {};
    (axum::http::StatusCode::OK, Html(template.render().unwrap()))
}
