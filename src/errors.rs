use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    extract::multipart::MultipartError,
};

pub enum AppError {
    Database(sqlx::Error),
    Multipart(MultipartError),
    NotFound,
    BadRequest(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<MultipartError> for AppError {
    fn from(err: MultipartError) -> Self {
        AppError::Multipart(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(e) => {
                eprintln!("[ERROR] Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Multipart(e) => {
                eprintln!("[ERROR] Multipart error: {}", e);
                (StatusCode::BAD_REQUEST, "Invalid upload".to_string())
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Paste not found".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, message).into_response()
    }
}
