use actix_web::{get, web, Responder};

use crate::errors::ApiError;
use crate::models::common::requests::StatisticsQuery;
use crate::models::common::responses::StatisticsResponse;
use crate::services::common::get_statistics_info;
use crate::utils::app::AppState;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(root).service(get_statistics);
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
    web::Json(serde_json::json!({
        "Status": "https://status.mcimirror.top",
        "Docs": [
            "https://mod.mcimirror.top/docs"
        ],
        "Github": "https://github.com/mcmod-info-mirror",
        "contact": {
            "Email": "z0z0r4@outlook.com",
            "QQ": "3531890582"
        }
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
