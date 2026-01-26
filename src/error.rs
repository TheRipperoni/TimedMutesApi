use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use diesel::r2d2;
use serde_json::json;

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Database error: {}", _0)]
    DatabaseError(diesel::result::Error),

    #[display(fmt = "Pool error: {}", _0)]
    PoolError(String),

    #[display(fmt = "Bskysdk error: {}", _0)]
    BskyError(String),

    #[display(fmt = "Not authorized")]
    Unauthorized,

    #[display(fmt = "Internal server error")]
    InternalError,

    #[display(fmt = "Not found")]
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

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(_) | AppError::InternalError | AppError::PoolError(_) => {
                HttpResponse::InternalServerError().json(json!({"error": self.to_string()}))
            }
            AppError::BskyError(e) => HttpResponse::BadRequest().json(json!({"error": e})),
            AppError::Unauthorized => {
                HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
            }
            AppError::NotFound => HttpResponse::NotFound().json(json!({"error": "Not found"})),
        }
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
