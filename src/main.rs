use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use color_eyre::Result;
use tower_livereload::LiveReloadLayer;

mod app_error;
mod content;

use app_error::AppError;
use content::{Article, Category};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let app = Router::new()
        .route("/", get(index))
        .route("/{category}/{slug}", get(show_article))
        .route("/static/{*file}", get(static_handler))
        .layer(LiveReloadLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    tracing::debug!("and we're off!");

    Ok(())
}

#[derive(Template, WebTemplate)]
#[template(path = "content.jinja")]
struct ArticleTemplate {
    article: Article,
    categories: Vec<Category>,
}

#[derive(Template, WebTemplate)]
#[template(path = "index.jinja")]
struct IndexTemplate {
    categories: Vec<Category>,
}

async fn show_article(
    Path((category, slug)): Path<(String, String)>,
) -> Result<ArticleTemplate, AppError> {
    let article = Article::from_path(category, slug).await?;
    let categories = content::get_categories().await?.clone();
    Ok(ArticleTemplate {
        article,
        categories,
    })
}

async fn index() -> Result<IndexTemplate, AppError> {
    let categories = content::get_categories().await?.clone();
    Ok(IndexTemplate { categories })
}

#[derive(rust_embed::Embed)]
#[folder = "static/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    StaticFile(path)
}
