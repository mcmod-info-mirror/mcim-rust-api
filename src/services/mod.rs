pub mod modrinth;
pub mod curseforge;
pub mod translate;
pub mod common;

#[derive(Debug)]
pub enum ServiceError {
    Database(mongodb::error::Error),
    InvalidData(String),
    NotFound,
    LogicalError(String),
}

impl From<mongodb::error::Error> for ServiceError {
    fn from(err: mongodb::error::Error) -> Self {
        ServiceError::Database(err)
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::Database(err) => write!(f, "Database error: {}", err),
            ServiceError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ServiceError::NotFound => write!(f, "Resource not found"),
            ServiceError::LogicalError(msg) => write!(f, "Logical error: {}", msg),
        }
    }
}