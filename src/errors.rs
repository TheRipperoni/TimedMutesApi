use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{error, HttpResponse, Result};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
enum UserError {
    #[display(fmt = "Validation error")]
    ValidationError,
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError,
}

impl error::ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::ValidationError {} => StatusCode::BAD_REQUEST,
            UserError::InternalError {} => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).finish()
    }
}
