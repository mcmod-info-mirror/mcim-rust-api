use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct StatisticsResponse {
    pub curseforge: Option<HashMap<String, u64>>,
    pub modrinth: Option<HashMap<String, u64>>,
    pub translate: Option<HashMap<String, u64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FilePlatform {
    Curseforge,
    Modrinth,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HotFile {
    pub platform: FilePlatform,
    /// 原站下载链接
    pub url: String,
    /// 文件大小（字节），数据库未收录时为 null
    pub size: Option<i64>,
    /// 当日请求次数
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FileStatisticsResponse {
    /// 统计日（UTC+8）
    pub date: NaiveDate,
    /// 请求次数需严格大于该值
    pub threshold: u64,
    /// 按请求次数降序
    pub files: Vec<HotFile>,
}
