use std::{collections::HashMap, sync::Mutex};

use askama::Template;
use comrak::{format_html_with_plugins, parse_document, Arena, Options, Plugins};

use crate::blog::posts::read::Post;

#[derive(Eq, PartialEq, Debug)]
struct TocItem {
    title: String,
    anchor: String,
    children: Vec<TocItem>,
}

#[derive(Eq, PartialEq, Debug)]
struct SinglePostView {
    title: String,
    toc: String,
    content: String,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct SinglePostTemplate {
    post: SinglePostView,
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

pub fn render_view(post: Post) -> SinglePostTemplate {
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

    SinglePostTemplate {
        post: SinglePostView {
            title: post.title,
            toc: format!("<ul>{}</ul>", toc_to_html(toc)),
            content: String::from_utf8(html).unwrap(),
        },
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;

    #[test]
    pub fn can_render_view() {
        let id = Uuid::new_v4();
        let post = Post {
            id,
            title: "Some post title".to_string(),
            content: "# Title
Some text
## Subtitle"
                .to_string(),
        };

        let rendered = render_view(post);

        assert_eq!(
            SinglePostView {
                title: "Some post title".to_string(),
                toc: "<ul><li><a href=\"#title\">Title</a><ul><li><a href=\"#subtitle\">Subtitle</a></li></ul></li></ul>".to_string(),
                content: "<h1 id=\"title\">Title</h1>\n<p>Some text</p>\n<h2 id=\"subtitle\">Subtitle</h2>".to_string()
            },
            rendered.post
        );
    }

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

        let expected = vec![
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
    #[test]
    pub fn can_convert_toc_to_html() {
        let toc = vec![
            TocItem {
                title: "a".to_string(),
                anchor: "a".to_string(),
                children: vec![],
            },
            TocItem {
                title: "b".to_string(),
                anchor: "b".to_string(),
                children: vec![
                    TocItem {
                        title: "c".to_string(),
                        anchor: "c".to_string(),
                        children: vec![],
                    },
                    TocItem {
                        title: "d".to_string(),
                        anchor: "d".to_string(),
                        children: vec![],
                    },
                ],
            },
        ];
        let html = toc_to_html(toc);

        // TODO: this function should include the outer <ul />
        assert_eq!("<li><a href=\"#a\">a</a></li><li><a href=\"#b\">b</a><ul><li><a href=\"#c\">c</a></li><li><a href=\"#d\">d</a></li></ul></li>", html);
    }
}
