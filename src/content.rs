use color_eyre::{Result, eyre::eyre};
use markdown::{Options, mdast::Node, to_html_with_options, to_mdast};
use serde::Deserialize;
use tokio::{fs, sync::OnceCell};

#[derive(Clone)]
pub struct Article {
    pub title: String,
    pub description: String,
    pub html: String,
    pub week: usize,
    pub slug: String,
}

impl Article {
    pub async fn from_slug(slug: String) -> Result<Self> {
        let source = fs::read_to_string(format!("content/{slug}.md")).await?;

        let mut opts = Options::gfm();
        opts.parse.constructs.frontmatter = true;

        let mut ast = to_mdast(&source, &opts.parse).unwrap();

        let mut yaml = None;

        if let Node::Root(root) = &mut ast {
            root.children.retain(|child| match child {
                Node::Yaml(node) => {
                    yaml = Some(node.value.clone());
                    false
                }
                // Node::Code(node) => {
                //     let html = highlight_code(&node.value, node.lang.as_deref())
                //         .wrap_err("failed to highlight code")
                //         .unwrap();
                //     *child = Node::Html(markdown::mdast::Html {
                //         value: html,
                //         position: node.position.clone(),
                //     });
                //     true
                // }
                _ => true,
            });
        }

        let meta: Frontmatter =
            serde_yaml::from_str(&yaml.ok_or(eyre!("no frontmatter in {slug}.md!"))?)?;
        let html = to_html_with_options(&source, &opts).unwrap(); // TODO: remove the unwrap

        Ok(Article {
            title: meta.title,
            description: meta.description,
            week: meta.week,
            html,
            slug,
        })
    }
}

static ARTICLES: OnceCell<Vec<Article>> = OnceCell::const_new();

pub async fn get_articles() -> Result<&'static Vec<Article>> {
    ARTICLES
        .get_or_try_init(|| async {
            let mut articles = Vec::new();
            let mut entries = fs::read_dir("content").await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }

                let slug = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .to_string();

                articles.push(Article::from_slug(slug).await?);
            }

            articles.sort_by_key(|a| a.week);

            Ok(articles)
        })
        .await
}

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    description: String,
    week: usize,
}
