use color_eyre::{Result, eyre::eyre};
use markdown::{Options, mdast::Node, to_html_with_options, to_mdast};
use serde::Deserialize;
use tokio::{fs, sync::OnceCell};

#[derive(Clone)]
pub struct Article {
    pub title: String,
    pub description: String,
    pub html: String,
    pub week: Option<usize>,
    pub slug: String,
    pub category: String,
}

impl Article {
    pub async fn from_path(category: String, slug: String) -> Result<Self> {
        let source = fs::read_to_string(format!("content/{category}/{slug}.md")).await?;

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
                _ => true,
            });
        }

        let meta: Frontmatter =
            serde_yaml::from_str(&yaml.ok_or(eyre!("no frontmatter in {category}/{slug}.md!"))?)?;
        let html = to_html_with_options(&source, &opts).unwrap();

        Ok(Article {
            title: meta.title,
            description: meta.description,
            week: meta.week,
            html,
            slug,
            category,
        })
    }

    pub fn path(&self) -> String {
        format!("/{}/{}", self.category, self.slug)
    }
}

#[derive(Clone)]
pub struct Category {
    pub name: String,
    pub display_name: String,
    pub articles: Vec<Article>,
}

static CATEGORIES: OnceCell<Vec<Category>> = OnceCell::const_new();

pub async fn get_categories() -> Result<&'static Vec<Category>> {
    CATEGORIES
        .get_or_try_init(|| async {
            let mut categories = Vec::new();
            let mut entries = fs::read_dir("content").await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let category_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .to_string();

                let display_name = category_name
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string() + &category_name[1..])
                    .unwrap_or_default();

                let mut articles = Vec::new();
                let mut sub_entries = fs::read_dir(&path).await?;

                while let Some(sub_entry) = sub_entries.next_entry().await? {
                    let sub_path = sub_entry.path();
                    if sub_path.extension().and_then(|s| s.to_str()) != Some("md") {
                        continue;
                    }

                    let slug = sub_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap()
                        .to_string();

                    articles.push(Article::from_path(category_name.clone(), slug).await?);
                }

                articles.sort_by(|a, b| {
                    a.week.unwrap_or(usize::MAX).cmp(&b.week.unwrap_or(usize::MAX))
                        .then_with(|| a.title.cmp(&b.title))
                });

                categories.push(Category {
                    name: category_name,
                    display_name,
                    articles,
                });
            }

            categories.sort_by(|a, b| a.display_name.cmp(&b.display_name));

            Ok(categories)
        })
        .await
}

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    description: String,
    week: Option<usize>,
}
