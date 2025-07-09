use mongodb::error::Error as MongoError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ServiceError {
    DatabaseError {
        message: String,
        source: Option<MongoError>,
    },
    NotFound {
        resource: String,
        detail: Option<String>,
    },
    InvalidInput {
        field: String,
        reason: String,
    },
    UnexpectedError(String),
    ExternalServiceError {
        service: String,
        message: String,
    },
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::DatabaseError { message, .. } => {
                write!(f, "Database error: {}", message)
            }
            ServiceError::NotFound { resource, detail } => {
                if let Some(detail) = detail {
                    write!(f, "Resource not found: {} ({})", resource, detail)
                } else {
                    write!(f, "Resource not found: {}", resource)
                }
            }
            ServiceError::InvalidInput { field, reason } => {
                write!(f, "Invalid input: {} - {}", field, reason)
            }
            ServiceError::UnexpectedError(message) => {
                write!(f, "Unexpected error: {}", message)
            }
            ServiceError::ExternalServiceError { service, message } => {
                write!(f, "External service error ({}): {}", service, message)
            }
        }
    }
}

impl StdError for ServiceError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ServiceError::DatabaseError { source, .. } => {
                source.as_ref().map(|e| e as &dyn std::error::Error)
            }
            ServiceError::NotFound { .. } => None,
            ServiceError::InvalidInput { .. } => None,
            ServiceError::UnexpectedError(_) => None,
            ServiceError::ExternalServiceError { .. } => None,
        }
    }
}

impl From<MongoError> for ServiceError {
    fn from(err: MongoError) -> Self {
        ServiceError::DatabaseError {
            message: err.to_string(),
            source: Some(err),
        }
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(err: serde_json::Error) -> Self {
        ServiceError::UnexpectedError(format!("JSON serialization error: {}", err))
    }
}
