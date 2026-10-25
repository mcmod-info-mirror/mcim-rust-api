use actix_web::{Responder, get, web};
use serde::Serialize;

use crate::errors::ApiError;
use crate::models::common::requests::{FileStatisticsQuery, StatisticsQuery};
use crate::models::common::responses::{FileStatisticsResponse, StatisticsResponse};
use crate::services::common::get_statistics_info;
use crate::services::file_stats::get_hot_files;
use crate::utils::app::AppState;

include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(root)
        .service(get_statistics)
        .service(get_file_statistics)
        .service(healthz);
}

#[derive(Serialize)]
struct VersionInfo {
    version: &'static str,
    commit: &'static str,
    branch: &'static str,
    build_time: &'static str,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Welcome message"),
    ),
    description = "Root endpoint for the MCIM API",
    tag = "Root"
)]
#[get("/")]
async fn root() -> impl Responder {
    let build_info = VersionInfo {
        version: VERSION,
        commit: GIT_COMMIT,
        branch: GIT_BRANCH,
        build_time: BUILD_TIME,
    };
    web::Json(serde_json::json!({
        "Home": "https://www.mcimirror.top",
        "Status": "https://www.mcimirror.top/status",
        "Docs": [
            "https://www.mcimirror.top/docs",
            "https://mod.mcimirror.top/docs"
        ],
        "Github": "https://github.com/mcmod-info-mirror",
        "contact": {
            "Email": "z0z0r4@outlook.com",
            "QQ": "3531890582"
        },
        "build": {
            "version": build_info.version,
            "commit": build_info.commit,
            "branch": build_info.branch,
            "build_time": build_info.build_time,
        }
    }))
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check message"),
    ),
    description = "Health check endpoint for the MCIM API",
)]
#[get("/healthz")]
async fn healthz() -> impl Responder {
    web::Json(serde_json::json!({
        "status": "OK"
    }))
}

#[utoipa::path(
    get,
    path = "/statistics",
    params(
        ("curseforge" = Option<bool>, Query, description = "Include CurseForge statistics"),
        ("modrinth" = Option<bool>, Query, description = "Include Modrinth statistics"),
        ("translate" = Option<bool>, Query, description = "Include translation statistics")
    ),
    responses(
        (status = 200, description = "Statistics retrieved successfully", body = StatisticsResponse),
        (status = 500, description = "Internal server error")
    ),
    description = "Get statistics for CurseForge and Modrinth",
    tag = "Common"
)]
#[get("/statistics")]
async fn get_statistics(
    query: web::Query<StatisticsQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    match get_statistics_info(
        query.curseforge.unwrap_or(true),
        query.modrinth.unwrap_or(true),
        query.translate.unwrap_or(true),
        &data.db,
    )
    .await
    {
        Ok(stats) => Ok(web::Json(stats)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    get,
    path = "/statistics_files",
    params(
        ("date" = Option<String>, Query, description = "Statistics day in YYYY-MM-DD (UTC+8); only today and yesterday are retained. Defaults to today", example = "2026-10-25")
    ),
    responses(
        (status = 200, description = "Files requested more than 10000 times on that day", body = FileStatisticsResponse),
        (status = 400, description = "Date is not retained"),
        (status = 500, description = "Internal server error")
    ),
    description = "Get files whose daily request count through the file CDN endpoints exceeds 10000, with original download URL and size",
    tag = "Common"
)]
#[get("/statistics_files")]
async fn get_file_statistics(
    query: web::Query<FileStatisticsQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let stats = get_hot_files(&data.db, &data.redis_pool, query.date).await?;
    Ok(web::Json(stats))
}
