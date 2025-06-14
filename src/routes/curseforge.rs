use actix_web::{get, post, web, HttpResponse, Responder};

use crate::config::AppState;
use crate::services::curseforge::CurseforgeService;
use crate::models::curseforge::requests::*;
use crate::errors::{ApiError, ServiceError};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/curseforge/v1")
            .service(get_mod)
            .service(get_files_by_ids)
            .service(get_mods)
            .service(get_mod_files)
            .service(get_file_download_url)
            .service(get_file)
            .service(get_fingerprints)
            .service(get_fingerprints_by_game_id)
            .service(get_categories),
    );
}

#[get("/mods/{mod_id}")]
async fn get_mod(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let mod_id = path.into_inner();

    let service = CurseforgeService::new(data.db.clone());

    match service.get_mod(mod_id).await {
        Ok(Some(mod_data)) => Ok(web::Json(mod_data)),
        Ok(None) => Err(ServiceError::NotFound {
            resource: "Mod".to_string(),
            detail: Some(mod_id.to_string()),
        }
        .into()),
        Err(e) => Err(e.into()),
    }
}

#[post("/mods")]
async fn get_mods(
    body: web::Json<ModsBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone());

    match service.get_mods(body.mod_ids.clone()).await {
        Ok(mods) => Ok(web::Json(mods)),
        Err(e) => Err(e.into()),
    }
}


#[get("/mods/{mod_id}/files")]
async fn get_mod_files(
    path: web::Path<i32>,
    query: web::Query<ModFilesQuery>,
    data: web::Data<AppState>,
)
-> Result<impl Responder, ApiError> {
    let mod_id = path.into_inner();
    let service = CurseforgeService::new(data.db.clone());

    match service.get_mod_files(
        mod_id,
        query.game_version.clone(),
        query.mod_loader_type,
        query.index,
        query.page_size,
    ).await {
        Ok(files) => Ok(web::Json(files)),
        Err(e) => Err(e.into())
    }
}


#[get("/mods/{mod_id}/files/{file_id}/download-url")]
async fn get_file_download_url(
    path: web::Path<(i32, i32)>,
    data: web::Data<AppState>,
)
-> Result<impl Responder, ApiError> {
    let (mod_id, file_id) = path.into_inner();

    let service = CurseforgeService::new(data.db.clone());

    match service.get_file_download_url(mod_id, file_id).await {
        Ok(url) => Ok(web::Json(url)),
        Err(e) => Err(e.into()),
    }
}

#[get("/mods/{mod_id}/files/{file_id}")]
async fn get_file(
    path: web::Path<(i32, i32)>,
    data: web::Data<AppState>,
)
-> Result<impl Responder, ApiError> {
    let (_, file_id) = path.into_inner();

    let service = CurseforgeService::new(data.db.clone());

    match service.get_file(file_id).await {
        Ok(file_data) => Ok(web::Json(file_data)),
        Err(e) => Err(e.into()),
    }
}

#[post("/mods/files")]
async fn get_files_by_ids(
    body: web::Json<FileIdsBody>,
    data: web::Data<AppState>,
)
-> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone());

    match service.get_files(body.file_ids.clone()).await {
        Ok(files) => Ok(web::Json(files)),
        Err(e) => Err(e.into()),
    }
}

#[post("/fingerprints")]
async fn get_fingerprints(
    body: web::Json<FingerprintsBody>,
    data: web::Data<AppState>,
)
-> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone());

    match service.get_fingerprints(body.fingerprints.clone(), None).await {
        Ok(fingerprint_result) => Ok(web::Json(fingerprint_result)),
        Err(e) => Err(e.into()),
    }
}

#[post("/fingerprints/{game_id}")]
async fn get_fingerprints_by_game_id(
    path: web::Path<i32>,
    body: web::Json<FingerprintsBody>,
    data: web::Data<AppState>,
)
-> Result<impl Responder, ApiError> {
    let game_id = path.into_inner();

    let service = CurseforgeService::new(data.db.clone());

    match service.get_fingerprints(body.fingerprints.clone(), Some(game_id)).await {
        Ok(fingerprint_result) => Ok(web::Json(fingerprint_result)),
        Err(e) => Err(e.into()),
    }
}

#[get("/categories")]
async fn get_categories(
    query: web::Query<CategoriesQuery>,
    data: web::Data<AppState>,
)
-> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone());

    match service.get_categories(query.game_id, query.class_id, query.class_only).await {
        Ok(categories) => Ok(web::Json(categories)),
        Err(e) => Err(e.into()),
    }
}
