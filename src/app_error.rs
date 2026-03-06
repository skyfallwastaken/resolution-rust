use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppError(color_eyre::Report);

impl From<color_eyre::Report> for AppError {
    fn from(value: color_eyre::Report) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
