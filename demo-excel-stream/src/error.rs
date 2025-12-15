use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    Excel(String),
    Config(String),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Excel(msg) => write!(f, "Excel error: {}", msg),
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for AppError {}

impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<rust_xlsxwriter::XlsxError> for AppError {
    fn from(err: rust_xlsxwriter::XlsxError) -> Self {
        AppError::Excel(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}

