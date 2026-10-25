//! 文件下载请求计数。
//!
//! 每个统计日（按 UTC+8 切分）一个 Redis 有序集合 `stats:files:{YYYYMMDD}`，
//! member 为文件标识，score 为当日请求次数。key 通过 EXPIREAT 在次日结束时过期，
//! 因此任意时刻最多只保留今天和昨天两个 key，内存占用随每日去重文件数线性增长且自动回收。

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Days, FixedOffset, NaiveDate, NaiveTime, Utc};
use futures::TryStreamExt;
use mongodb::bson::doc;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use serde::Deserialize;

use crate::db::database::get_database_name;
use crate::errors::ServiceError;
use crate::models::common::responses::{FilePlatform, FileStatisticsResponse, HotFile};

/// 单日请求次数必须严格大于该值才会出现在统计结果中
pub const HOT_FILE_THRESHOLD: u64 = 10_000;

/// 统计日按 UTC+8 切分
const STATS_UTC_OFFSET_SECS: i32 = 8 * 3600;
/// 计数保留的统计日数量（含当天）：今天 + 昨天
const RETENTION_DAYS: u64 = 2;
/// 超出该长度的文件名不计数，避免异常请求放大 member 体积
const MAX_FILE_NAME_LEN: usize = 255;
const MAX_MODRINTH_ID_LEN: usize = 16;

const CURSEFORGE_ORIGIN: &str = "https://edge.forgecdn.net";
const MODRINTH_ORIGIN: &str = "https://cdn.modrinth.com";

/// 当前统计日（UTC+8）
pub fn stats_today() -> NaiveDate {
    let offset = FixedOffset::east_opt(STATS_UTC_OFFSET_SECS).expect("valid UTC offset");
    Utc::now().with_timezone(&offset).date_naive()
}

/// 统计日对应的 Redis key
pub fn daily_key(date: NaiveDate) -> String {
    format!("stats:files:{}", date.format("%Y%m%d"))
}

/// `date` 的计数 key 在其后第 RETENTION_DAYS 天 0 点（UTC+8）过期
fn expire_at(date: NaiveDate) -> i64 {
    let midnight = date
        .checked_add_days(Days::new(RETENTION_DAYS))
        .expect("statistics date out of range")
        .and_time(NaiveTime::MIN);
    midnight.and_utc().timestamp() - i64::from(STATS_UTC_OFFSET_SECS)
}

fn valid_file_name(file_name: &str) -> bool {
    !file_name.is_empty() && file_name.len() <= MAX_FILE_NAME_LEN
}

fn valid_modrinth_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_MODRINTH_ID_LEN
        && id.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// `/files/{id1}/{id2}/{file_name}` 的计数 member：`c:{file_id}:{file_name}`。
/// file_id = id1 * 1000 + id2，同时合并 `080` 与 `80` 这类写法。非法路径返回 None（不计数）。
pub fn curseforge_member(id1: &str, id2: &str, file_name: &str) -> Option<String> {
    let id1: u32 = id1.parse().ok()?;
    let id2: u32 = id2.parse().ok()?;
    if id2 >= 1000 || !valid_file_name(file_name) {
        return None;
    }
    let file_id = id1.checked_mul(1000)?.checked_add(id2)?;
    Some(format!("c:{}:{}", file_id, file_name))
}

/// `/data/{project_id}/versions/{version_id}/{file_name}` 的计数 member：
/// `m:{project_id}:{version_id}:{file_name}`。非法路径返回 None（不计数）。
pub fn modrinth_member(project_id: &str, version_id: &str, file_name: &str) -> Option<String> {
    if !valid_modrinth_id(project_id)
        || !valid_modrinth_id(version_id)
        || !valid_file_name(file_name)
    {
        return None;
    }
    Some(format!("m:{}:{}:{}", project_id, version_id, file_name))
}

/// 在后台累加当日计数，不阻塞重定向响应；Redis 失败只记录日志。
pub fn record_file_request(redis: &Arc<MultiplexedConnection>, member: String) {
    let mut conn = redis.as_ref().clone();
    let date = stats_today();
    actix_web::rt::spawn(async move {
        let key = daily_key(date);
        let result: redis::RedisResult<()> = redis::pipe()
            .zincr(&key, member, 1)
            .ignore()
            .expire_at(&key, expire_at(date))
            .ignore()
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::warn!("Failed to record file request in {}: {}", key, e);
        }
    });
}

enum ParsedMember<'a> {
    Curseforge {
        file_id: i64,
        file_name: &'a str,
    },
    Modrinth {
        project_id: &'a str,
        version_id: &'a str,
        file_name: &'a str,
    },
}

fn parse_member(member: &str) -> Option<ParsedMember<'_>> {
    let (kind, rest) = member.split_once(':')?;
    match kind {
        "c" => {
            let (file_id, file_name) = rest.split_once(':')?;
            Some(ParsedMember::Curseforge {
                file_id: file_id.parse().ok()?,
                file_name,
            })
        }
        "m" => {
            let mut parts = rest.splitn(3, ':');
            Some(ParsedMember::Modrinth {
                project_id: parts.next()?,
                version_id: parts.next()?,
                file_name: parts.next()?,
            })
        }
        _ => None,
    }
}

#[derive(Deserialize)]
struct CurseforgeFileSize {
    #[serde(rename = "_id")]
    id: i64,
    #[serde(rename = "fileLength")]
    file_length: Option<i64>,
}

#[derive(Deserialize)]
struct ModrinthVersionFiles {
    #[serde(rename = "_id")]
    id: String,
    #[serde(default)]
    files: Vec<ModrinthFileSize>,
}

#[derive(Deserialize)]
struct ModrinthFileSize {
    filename: String,
    size: i64,
}

async fn curseforge_file_sizes(
    db: &mongodb::Client,
    file_ids: &[i64],
) -> Result<HashMap<i64, i64>, ServiceError> {
    if file_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let collection = db
        .database(get_database_name().as_str())
        .collection::<CurseforgeFileSize>("curseforge_files");
    let files: Vec<CurseforgeFileSize> = collection
        .find(doc! { "_id": { "$in": file_ids } })
        .projection(doc! { "fileLength": 1 })
        .await
        .map_err(|e| ServiceError::DatabaseError {
            message: String::from("Failed to fetch curseforge file sizes"),
            source: Some(e),
        })?
        .try_collect()
        .await
        .map_err(|e| ServiceError::DatabaseError {
            message: String::from("Failed to fetch curseforge file sizes"),
            source: Some(e),
        })?;
    Ok(files
        .into_iter()
        .filter_map(|f| Some((f.id, f.file_length?)))
        .collect())
}

/// 返回 (version_id, filename) -> size
async fn modrinth_file_sizes(
    db: &mongodb::Client,
    version_ids: &[&str],
) -> Result<HashMap<(String, String), i64>, ServiceError> {
    if version_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let collection = db
        .database(get_database_name().as_str())
        .collection::<ModrinthVersionFiles>("modrinth_versions");
    let versions: Vec<ModrinthVersionFiles> = collection
        .find(doc! { "_id": { "$in": version_ids } })
        .projection(doc! { "files.filename": 1, "files.size": 1 })
        .await
        .map_err(|e| ServiceError::DatabaseError {
            message: String::from("Failed to fetch modrinth file sizes"),
            source: Some(e),
        })?
        .try_collect()
        .await
        .map_err(|e| ServiceError::DatabaseError {
            message: String::from("Failed to fetch modrinth file sizes"),
            source: Some(e),
        })?;
    Ok(versions
        .into_iter()
        .flat_map(|v| {
            let version_id = v.id;
            v.files
                .into_iter()
                .map(move |f| ((version_id.clone(), f.filename), f.size))
        })
        .collect())
}

/// 查询某统计日（默认今天）请求次数超过 HOT_FILE_THRESHOLD 的文件，按次数降序。
pub async fn get_hot_files(
    db: &mongodb::Client,
    redis: &Arc<MultiplexedConnection>,
    date: Option<NaiveDate>,
) -> Result<FileStatisticsResponse, ServiceError> {
    let today = stats_today();
    let oldest = today - Days::new(RETENTION_DAYS - 1);
    let date = date.unwrap_or(today);
    if date > today || date < oldest {
        return Err(ServiceError::InvalidInput {
            field: String::from("date"),
            reason: format!("only {} to {} (UTC+8) are retained", oldest, today),
        });
    }

    let mut conn = redis.as_ref().clone();
    let entries: Vec<(String, f64)> = conn
        .zrevrangebyscore_withscores(daily_key(date), "+inf", format!("({}", HOT_FILE_THRESHOLD))
        .await
        .map_err(|e| ServiceError::ExternalServiceError {
            service: String::from("Redis"),
            message: format!("Failed to read file statistics: {}", e),
        })?;

    let parsed: Vec<(ParsedMember, u64)> = entries
        .iter()
        .filter_map(|(member, score)| Some((parse_member(member)?, *score as u64)))
        .collect();

    let mut curseforge_ids = Vec::new();
    let mut modrinth_version_ids = Vec::new();
    for (member, _) in &parsed {
        match member {
            ParsedMember::Curseforge { file_id, .. } => curseforge_ids.push(*file_id),
            ParsedMember::Modrinth { version_id, .. } => modrinth_version_ids.push(*version_id),
        }
    }
    curseforge_ids.sort_unstable();
    curseforge_ids.dedup();
    modrinth_version_ids.sort_unstable();
    modrinth_version_ids.dedup();

    let curseforge_sizes = curseforge_file_sizes(db, &curseforge_ids).await?;
    let modrinth_sizes = modrinth_file_sizes(db, &modrinth_version_ids).await?;

    let files = parsed
        .into_iter()
        .map(|(member, count)| match member {
            ParsedMember::Curseforge { file_id, file_name } => HotFile {
                platform: FilePlatform::Curseforge,
                url: format!(
                    "{}/files/{}/{}/{}",
                    CURSEFORGE_ORIGIN,
                    file_id / 1000,
                    file_id % 1000,
                    urlencoding::encode(file_name)
                ),
                size: curseforge_sizes.get(&file_id).copied(),
                count,
            },
            ParsedMember::Modrinth {
                project_id,
                version_id,
                file_name,
            } => HotFile {
                platform: FilePlatform::Modrinth,
                url: format!(
                    "{}/data/{}/versions/{}/{}",
                    MODRINTH_ORIGIN,
                    project_id,
                    version_id,
                    urlencoding::encode(file_name)
                ),
                size: modrinth_sizes
                    .get(&(version_id.to_string(), file_name.to_string()))
                    .copied(),
                count,
            },
        })
        .collect();

    Ok(FileStatisticsResponse {
        date,
        threshold: HOT_FILE_THRESHOLD,
        files,
    })
}
