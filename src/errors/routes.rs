use actix_web::{HttpResponse, ResponseError};
use std::error::Error as StdError;
use std::fmt::Display;

use crate::errors::ServiceError;
use crate::models::ErrorResponse;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
}

impl ApiError {
    pub fn to_response(&self) -> HttpResponse {
        let resp = match self {
            ApiError::NotFound(msg) => ErrorResponse {
                code: 404,
                error: "Not Found".to_string(),
                detail: Some(msg.clone()),
            },
            ApiError::BadRequest(msg) => ErrorResponse {
                code: 400,
                error: "Bad Request".to_string(),
                detail: Some(msg.clone()),
            },
            ApiError::InternalServerError(msg) => ErrorResponse {
                code: 500,
                error: "Internal Server Error".to_string(),
                detail: Some(msg.clone()),
            },
        };
        HttpResponse::build(actix_web::http::StatusCode::from_u16(resp.code).unwrap()).json(resp)
    }
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::DatabaseError { message, source: _ } => {
                ApiError::InternalServerError(format!("Database error: {}", message))
            }
            ServiceError::NotFound { resource, detail } => {
                let msg = match detail {
                    Some(d) => format!("{} not found: {}", resource, d),
                    None => format!("{} not found", resource),
                };
                ApiError::NotFound(msg)
            }
            ServiceError::InvalidInput { field, reason } => {
                ApiError::BadRequest(format!("{}: {}", field, reason))
            }
            ServiceError::UnexpectedError(message) => {
                ApiError::InternalServerError(format!("Unexpected error: {}", message))
            }
            ServiceError::ExternalServiceError { service, message } => {
                ApiError::InternalServerError(format!("{} service error: {}", service, message))
            }
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
        }
    }
}

impl StdError for ApiError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ApiError::NotFound(_) => None,
            ApiError::BadRequest(_) => None,
            ApiError::InternalServerError(_) => None,
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        self.to_response()
    }
}

pub fn map_actix_error(err: actix_web::Error) -> ApiError {
    if let Some(path_err) = err.as_error::<actix_web::error::PathError>() {
        return ApiError::BadRequest(format!("Invalid path parameter: {}", path_err));
    }

    if let Some(query_err) = err.as_error::<actix_web::error::QueryPayloadError>() {
        return ApiError::BadRequest(format!("Invalid query parameter: {}", query_err));
    }

    if let Some(json_err) = err.as_error::<actix_web::error::JsonPayloadError>() {
        return ApiError::BadRequest(format!("Invalid JSON payload: {}", json_err));
    }

    if let Some(payload_err) = err.as_error::<actix_web::error::PayloadError>() {
        return ApiError::BadRequest(format!("Payload too large: {}", payload_err));
    }

    if let Some(header_err) = err.as_error::<actix_web::http::header::InvalidHeaderValue>() {
        return ApiError::BadRequest(format!("Invalid header value: {}", header_err));
    }

    // 兜底
    ApiError::InternalServerError("Internal server error".to_string())
}
