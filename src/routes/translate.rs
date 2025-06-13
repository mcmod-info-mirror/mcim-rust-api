use actix_web::{get, post, web, HttpResponse, Responder};

use crate::config::AppState;
use crate::models::translate::requests::{
    CurseForgeTranslationRequest, CurseforgeQuery, ModrinthQuery, ModrinthTranslationRequest,
};
use crate::services::translate::{CurseForgeService, ModrinthService};


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/translate")
            .service(get_modrinth_translation)
            .service(get_modrinth_translation_deprecated)
            .service(get_curseforge_translation)
            .service(get_curseforge_translation_deprecated)
            .service(get_modrinth_translation_batch)
            .service(get_curseforge_translation_batch)
    );
}

#[get("/modrinth/{project_id}")]
async fn get_modrinth_translation(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let project_id = path.into_inner();
    if project_id.is_empty() {
        return HttpResponse::BadRequest().body("Project ID cannot be empty");
    }

    let service = ModrinthService::new(data.db.clone());

    match service.get_translation(&project_id).await {
        Ok(Some(translation)) => HttpResponse::Ok().json(translation),
        Ok(None) => HttpResponse::NotFound().body("Translation not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

// Deprecated
#[get("/modrinth")]
async fn get_modrinth_translation_deprecated(
    query: web::Query<ModrinthQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    let project_id = query.project_id.clone();
    if project_id.is_empty() {
        return HttpResponse::BadRequest().body("Project ID cannot be empty");
    }

    let service = ModrinthService::new(data.db.clone());

    match service.get_translation(&project_id).await {
        Ok(Some(translation)) => HttpResponse::Ok().json(translation),
        Ok(None) => HttpResponse::NotFound().body("Translation not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[get("/curseforge/{mod_id}")]
async fn get_curseforge_translation(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mod_id = path.into_inner();

    let service = CurseForgeService::new(data.db.clone());

    match service.get_translation(mod_id).await {
        Ok(Some(translation)) => HttpResponse::Ok().json(translation),
        Ok(None) => HttpResponse::NotFound().body("Translation not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

// Deprecated
#[get("/curseforge")]
async fn get_curseforge_translation_deprecated(
    query: web::Query<CurseforgeQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mod_id = query.mod_id;

    let service = CurseForgeService::new(data.db.clone());

    match service.get_translation(mod_id).await {
        Ok(Some(translation)) => HttpResponse::Ok().json(translation),
        Ok(None) => HttpResponse::NotFound().body("Translation not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[post("/modrinth")]
async fn get_modrinth_translation_batch(
    data: web::Data<AppState>,
    body: web::Json<ModrinthTranslationRequest>,
) -> impl Responder {
    let project_ids = body.project_ids.clone();
    let service = ModrinthService::new(data.db.clone());

    match service.get_translations_batch(project_ids).await {
        Ok(translation) => HttpResponse::Created().json(translation),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}

#[post("/curseforge")]
async fn get_curseforge_translation_batch(
    data: web::Data<AppState>,
    body: web::Json<CurseForgeTranslationRequest>,
) -> impl Responder {
    let mod_ids = body.modids.clone();
    let service = CurseForgeService::new(data.db.clone());

    match service.get_translations_batch(mod_ids).await {
        Ok(translation) => HttpResponse::Created().json(translation),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service error: {:?}", e)),
    }
}
