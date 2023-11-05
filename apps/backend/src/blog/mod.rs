use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use askama::Template;
use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
};
use comrak::{format_html_with_plugins, parse_document, Arena, Options, Plugins};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

use self::posts::{read, Post, Repository};

pub mod posts;

#[derive(Eq, PartialEq, Debug)]
struct TocItem {
    title: String,
    anchor: String,
    children: Vec<TocItem>,
}

struct SinglePostView {
    title: String,
    toc: String,
    content: String,
}

struct LatestPostView {
    id: Uuid,
    title: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<LatestPostView>,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: SinglePostView,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Heading {
    content: String,
    slug: String,
    level: u8,
}

struct HeadingAdapter {
    headings: Mutex<Vec<Heading>>,
}

impl comrak::adapters::HeadingAdapter for HeadingAdapter {
    fn enter(
        &self,
        output: &mut dyn std::io::Write,
        heading: &comrak::adapters::HeadingMeta,
        _sourcepos: Option<comrak::nodes::Sourcepos>,
    ) -> std::io::Result<()> {
        let slug = slug::slugify(&heading.content);
        self.headings.lock().unwrap().push(Heading {
            content: heading.content.clone(),
            slug: slug.clone(),
            level: heading.level,
        });
        write!(output, "<h{} id=\"{}\">", heading.level, slug)
    }

    fn exit(
        &self,
        output: &mut dyn std::io::Write,
        heading: &comrak::adapters::HeadingMeta,
    ) -> std::io::Result<()> {
        write!(output, "</h{}>", heading.level)
    }
}

fn generate_toc(headings: Vec<Heading>) -> Vec<TocItem> {
    let min_level = headings.iter().map(|x| x.level).min().unwrap_or(0);

    let mut output: Vec<TocItem> = vec![];
    let mut lvl_map: HashMap<String, u8> = HashMap::new();

    for h in headings.into_iter().map(|x| Heading {
        level: x.level - min_level,
        ..x
    }) {
        lvl_map.insert(h.content.clone(), h.level);

        let last = output.last_mut();
        let slug = h.slug.clone();

        if let Some(mut last) = last {
            let lvl = *lvl_map.get(&last.title).unwrap();
            if lvl < h.level {
                loop {
                    let len = last.children.len();

                    if len > 0 {
                        let last_ = last.children.last_mut().unwrap();
                        let lvl = *lvl_map.get(&last_.title).unwrap();

                        if lvl < h.level {
                            last = last.children.last_mut().unwrap();
                            continue;
                        }
                    }

                    break;
                }

                last.children.push(TocItem {
                    title: h.content.clone(),
                    anchor: h.slug,
                    children: vec![],
                });
            } else {
                output.push(TocItem {
                    title: h.content.clone(),
                    anchor: slug,
                    children: vec![],
                });
            }
        } else {
            output.push(TocItem {
                title: h.content.clone(),
                anchor: slug,
                children: vec![],
            });
        }
    }

    output
}

pub async fn route_posts_id(
    Path(id): Path<Uuid>,
    State(blog): State<Arc<Blog>>,
) -> impl IntoResponse {
    let post = blog.posts.single(id).await.unwrap().unwrap();

    let arena = Arena::new();

    let root = parse_document(&arena, &post.content, &Options::default());

    let mut plugins = Plugins::default();
    let heading_adapter = HeadingAdapter {
        headings: Mutex::new(vec![]),
    };
    plugins.render.heading_adapter = Some(&heading_adapter);

    let mut html = vec![];
    format_html_with_plugins(root, &Options::default(), &mut html, &plugins).unwrap();

    let toc = generate_toc(heading_adapter.headings.lock().unwrap().clone());

    let template = PostTemplate {
        post: SinglePostView {
            title: post.title,
            toc: format!("<ul>{}</ul>", toc_to_html(toc)),
            content: String::from_utf8(html).unwrap(),
        },
    };

    (axum::http::StatusCode::OK, Html(template.render().unwrap()))
}

fn toc_to_html(toc: Vec<TocItem>) -> String {
    let mut result = String::new();

    for item in toc {
        let children = if item.children.is_empty() {
            String::new()
        } else {
            format!("<ul>{}</ul>", toc_to_html(item.children))
        };

        result += &format!(
            "<li><a href=\"#{}\">{}</a>{}</li>",
            item.anchor, item.title, children
        );
    }

    result
}

impl Blog {
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        Self {
            posts: read::Read::new(db_pool),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn can_generate_toc() {
        let headings = vec![
            Heading {
                content: "abc1".to_string(),
                slug: "abc1".to_string(),
                level: 2,
            },
            Heading {
                content: "abc2".to_string(),
                slug: "abc1".to_string(),
                level: 2,
            },
            Heading {
                content: "abc3".to_string(),
                slug: "abc1".to_string(),
                level: 3,
            },
            Heading {
                content: "abc4".to_string(),
                slug: "abc1".to_string(),
                level: 4,
            },
            Heading {
                content: "abc5".to_string(),
                slug: "abc1".to_string(),
                level: 4,
            },
            Heading {
                content: "abc6".to_string(),
                slug: "abc1".to_string(),
                level: 5,
            },
            Heading {
                content: "abc7".to_string(),
                slug: "abc1".to_string(),
                level: 2,
            },
        ];

        let mut expected = vec![
            TocItem {
                title: "abc1".to_string(),
                anchor: "abc1".to_string(),
                children: vec![],
            },
            TocItem {
                title: "abc2".to_string(),
                anchor: "abc1".to_string(),
                children: vec![TocItem {
                    title: "abc3".to_string(),
                    anchor: "abc1".to_string(),
                    children: vec![
                        TocItem {
                            title: "abc4".to_string(),
                            anchor: "abc1".to_string(),
                            children: vec![],
                        },
                        TocItem {
                            title: "abc5".to_string(),
                            anchor: "abc1".to_string(),
                            children: vec![TocItem {
                                title: "abc6".to_string(),
                                anchor: "abc1".to_string(),
                                children: vec![],
                            }],
                        },
                    ],
                }],
            },
            TocItem {
                title: "abc7".to_string(),
                anchor: "abc1".to_string(),
                children: vec![],
            },
        ];

        let toc = generate_toc(headings);

        pretty_assertions::assert_eq!(expected, toc);
    }
}
