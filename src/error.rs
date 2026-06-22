use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::Display;
use diesel::r2d2;
use serde_json::json;

#[derive(Debug, Display)]
pub enum AppError {
    #[display("Database error: {_0}")]
    DatabaseError(diesel::result::Error),

    #[display("Pool error: {_0}")]
    PoolError(String),

    #[display("Bskysdk error: {_0}")]
    BskyError(String),

    #[display("Not authorized")]
    Unauthorized,

    #[display("Internal server error")]
    InternalError,

    #[display("Not found")]
    NotFound,
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::DatabaseError(err) => Some(err),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(_) | AppError::InternalError | AppError::PoolError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::BskyError(e) => (StatusCode::BAD_REQUEST, e),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
        };

        let body = axum::Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<r2d2::Error> for AppError {
    fn from(err: r2d2::Error) -> Self {
        AppError::PoolError(err.to_string())
    }
}
