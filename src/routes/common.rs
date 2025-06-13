use actix_web::{get, web, HttpResponse, Responder};

use crate::config::AppState;
use crate::services::common::get_statistics_info;
use crate::models::common::requests::StatisticsQuery;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(root)
    .service(get_statistics);
}

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the MCIM Translation API!")
}

#[get("/statistics")]
async fn get_statistics(
    query: web::Query<StatisticsQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    match get_statistics_info(
        query.curseforge.unwrap_or(true),
        query.modrinth.unwrap_or(true),
        query.translate.unwrap_or(true),
        &data.db
    )
    .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => {
            eprintln!("Error fetching statistics: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to fetch statistics")
        }
    }
}
