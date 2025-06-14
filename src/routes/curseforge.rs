use actix_web::{get, post, web, HttpResponse, Responder};

use crate::config::AppState;
use crate::services::curseforge::CurseforgeService;
use crate::models::curseforge::requests::*;

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
)
-> impl Responder {
    let mod_id = path.into_inner();
    if mod_id <= 0 {
        return HttpResponse::BadRequest().body("Mod ID must be a positive integer");
    }

    let service = CurseforgeService::new(data.db.clone());

    match service.get_mod(mod_id).await {
        Ok(Some(mod_data)) => HttpResponse::Ok().json(mod_data),
        Ok(None) => HttpResponse::NotFound().body("Mod not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[post("/mods")]
async fn get_mods(
    body: web::Json<ModsBody>,
    data: web::Data<AppState>,
)
-> impl Responder {
    let service = CurseforgeService::new(data.db.clone());

    match service.get_mods(body.mod_ids.clone()).await {
        Ok(mods) if mods.data.is_empty() => HttpResponse::NotFound().body("No mods found"),
        Ok(mods) => HttpResponse::Ok().json(mods),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}


#[get("/mods/{mod_id}/files")]
async fn get_mod_files(
    path: web::Path<i32>,
    query: web::Query<ModFilesQuery>,
    data: web::Data<AppState>,
)
-> impl Responder {
    let mod_id = path.into_inner();
    let service = CurseforgeService::new(data.db.clone());

    match service.get_mod_files(
        mod_id,
        query.game_version.clone(),
        query.mod_loader_type,
        query.index,
        query.page_size,
    ).await {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}


#[get("/mods/{mod_id}/files/{file_id}/download-url")]
async fn get_file_download_url(
    path: web::Path<(i32, i32)>,
    data: web::Data<AppState>,
)
-> impl Responder {
    let (mod_id, file_id) = path.into_inner();
    if mod_id <= 0 || file_id <= 0 {
        return HttpResponse::BadRequest().body("Mod ID and File ID must be positive integers");
    }

    let service = CurseforgeService::new(data.db.clone());

    match service.get_file_download_url(mod_id, file_id).await {
        Ok(url) => HttpResponse::Ok().json(url),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[get("/mods/{mod_id}/files/{file_id}")]
async fn get_file(
    path: web::Path<(i32, i32)>,
    data: web::Data<AppState>,
)
-> impl Responder {
    let (mod_id, file_id) = path.into_inner();
    if mod_id <= 0 || file_id <= 0 {
        return HttpResponse::BadRequest().body("Mod ID and File ID must be positive integers");
    }

    let service = CurseforgeService::new(data.db.clone());

    match service.get_file(file_id).await {
        Ok(Some(file_data)) => HttpResponse::Ok().json(file_data),
        Ok(None) => HttpResponse::NotFound().body("File not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[post("/mods/files")]
async fn get_files_by_ids(
    body: web::Json<FileIdsBody>,
    data: web::Data<AppState>,
)
-> impl Responder {
    if body.file_ids.is_empty() {
        return HttpResponse::BadRequest().body("File IDs cannot be empty");
    }

    let service = CurseforgeService::new(data.db.clone());

    match service.get_files(body.file_ids.clone()).await {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[post("/fingerprints")]
async fn get_fingerprints(
    body: web::Json<FingerprintsBody>,
    data: web::Data<AppState>,
)
-> impl Responder {
    if body.fingerprints.is_empty() {
        return HttpResponse::BadRequest().body("Fingerprints cannot be empty");
    }

    let service = CurseforgeService::new(data.db.clone());

    match service.get_fingerprints(body.fingerprints.clone(), None).await {
        Ok(fingerprint_result) => HttpResponse::Ok().json(fingerprint_result),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[post("/fingerprints/{game_id}")]
async fn get_fingerprints_by_game_id(
    path: web::Path<i32>,
    body: web::Json<FingerprintsBody>,
    data: web::Data<AppState>,
)
-> impl Responder {
    let game_id = path.into_inner();
    if game_id <= 0 {
        return HttpResponse::BadRequest().body("Game ID must be a positive integer");
    }

    if body.fingerprints.is_empty() {
        return HttpResponse::BadRequest().body("Fingerprints cannot be empty");
    }

    let service = CurseforgeService::new(data.db.clone());

    match service.get_fingerprints(body.fingerprints.clone(), Some(game_id)).await {
        Ok(fingerprint_result) => HttpResponse::Ok().json(fingerprint_result),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[get("/categories")]
async fn get_categories(
    query: web::Query<CategoriesQuery>,
    data: web::Data<AppState>,
)
-> impl Responder {
    let service = CurseforgeService::new(data.db.clone());

    match service.get_categories(query.game_id, query.class_id, query.class_only).await {
        Ok(categories) => HttpResponse::Ok().json(categories),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}
