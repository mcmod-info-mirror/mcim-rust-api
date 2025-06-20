use crate::errors::ApiError;

use actix_web::HttpResponse;
use futures::Future;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;

/// 通用缓存包装器
pub async fn cacheable_json<T, F>(
    pool: Arc<MultiplexedConnection>,
    key: String,
    ttl: u64,
    handler: F,
) -> Result<HttpResponse, ApiError>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
    F: FnOnce() -> Pin<Box<dyn Future<Output = Result<T, ApiError>> + Send + 'static>>,
{
    let mut conn = pool.as_ref().clone();
    let mut use_cache = true;

    // 检查缓存是否存在
    match conn.get::<_, Option<String>>(&key).await {
        Ok(Some(cached)) => {
            return Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(cached));
        }
        Ok(None) => {} // 没有缓存，继续执行
        Err(e) => {
            log::error!("Redis get error: {}", e);
            use_cache = false;
        }
    }

    // 执行原始逻辑
    match handler().await {
        Ok(data) => {
            // 只有成功时才缓存和返回
            let json_result: String = serde_json::to_string(&data).map_err(|e| {
                log::error!("JSON serialize error: {}", e);
                ApiError::InternalServerError("Serialization error".to_string())
            })?;

            if use_cache {
                if let Err(e) = conn
                    .set_ex::<&str, &str, ()>(&key, json_result.as_str(), ttl)
                    .await
                {
                    log::error!("Redis set error: {}", e);
                }
            }

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(json_result))
        }
        Err(e) => {
            // 错误时直接返回，不缓存
            Err(e)
        }
    }
}

use md5::compute;

pub fn create_key(method: String, path: String, query: String) -> String {
    let base_key = if query.is_empty() {
        format!("{}:{}", method, path)
    } else {
        format!("{}:{}?{}", method, path, query)
    };

    // Hash the key using MD5
    let result = compute(base_key.as_bytes());
    // Convert to hex string
    format!("cache:{:x}", result)
}
