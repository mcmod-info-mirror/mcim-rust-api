use actix_web::{get, post, web, Responder};

use crate::models::translate::requests::{
    CurseForgeTranslationRequest, CurseforgeQuery, ModrinthQuery, ModrinthTranslationRequest,
};
use crate::models::translate::responses::{
    CurseForgeTranslationResponse, ModrinthTranslationResponse,
};
use crate::services::translate::{CurseForgeService, ModrinthService};
use crate::utils::app::AppState;
use crate::errors::ApiError;

#[allow(deprecated)]
pub mod deprecated_routes {
    use super::*;

    #[utoipa::path(
        get,
        path = "/translate/modrinth",
        params(
            ("project_id" = String, Query, description = "Project ID of the Modrinth project")
        ),
        responses(
            (status = 200, description = "Translation found", body = ModrinthTranslationResponse),
            (status = 404, description = "Translation not found"),
            (status = 500, description = "Internal server error")
        ),
        description = "Get Modrinth translation by project ID",
        tag = "Translate",
    )]
    #[deprecated]
    #[get("/modrinth")]
    pub(super) async fn get_modrinth_translation_deprecated(
        query: web::Query<ModrinthQuery>,
        data: web::Data<AppState>,
    ) -> Result<impl Responder, ApiError> {
        let project_id = query.project_id.clone();

        let service = ModrinthService::new(data.db.clone());

        match service.get_translation(&project_id).await {
            Ok(translation) => Ok(web::Json(translation)),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    #[utoipa::path(
        get,
        path = "/translate/curseforge",
        params(
            ("modId" = i32, Query, description = "Mod ID of the CurseForge mod")
        ),
        responses(
            (status = 200, description = "Translation found", body = CurseForgeTranslationResponse),
            (status = 404, description = "Translation not found"),
            (status = 500, description = "Internal server error")
        ),
        description = "Get CurseForge translation by mod ID",
        tag = "Translate",
    )]
    #[deprecated]
    #[get("/curseforge")]
    pub(super) async fn get_curseforge_translation_deprecated(
        query: web::Query<CurseforgeQuery>,
        data: web::Data<AppState>,
    ) -> Result<impl Responder, ApiError> {
        let mod_id = query.mod_id;

        let service = CurseForgeService::new(data.db.clone());

        match service.get_translation(mod_id).await {
            Ok(translation) => Ok(web::Json(translation)),
            Err(e) => Err(ApiError::from(e)),
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    use deprecated_routes::*;

    cfg.service(
        web::scope("/translate")
            .service(get_modrinth_translation)
            .service(get_modrinth_translation_deprecated)
            .service(get_curseforge_translation)
            .service(get_curseforge_translation_deprecated)
            .service(get_modrinth_translation_batch)
            .service(get_curseforge_translation_batch),
    );
}

#[utoipa::path(
    get,
    path = "/translate/modrinth/{project_id}",
    params(
        ("project_id" = String, Path, description = "Project ID of the Modrinth project")
    ),
    responses(
        (status = 200, description = "Translation found", body = ModrinthTranslationResponse),
        (status = 404, description = "Translation not found"),
        (status = 500, description = "Internal server error")
    ),
    description = "Get Modrinth translation by project ID",
    tag = "Translate",
)]
#[get("/modrinth/{project_id}")]
async fn get_modrinth_translation(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let project_id = path.into_inner();

    let service = ModrinthService::new(data.db.clone());

    match service.get_translation(&project_id).await {
        Ok(translation) => Ok(web::Json(translation)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/translate/curseforge/{mod_id}",
    params(
        ("mod_id" = i32, Path, description = "Mod ID of the CurseForge mod")
    ),
    responses(
        (status = 200, description = "Translation found", body = CurseForgeTranslationResponse),
        (status = 404, description = "Translation not found"),
        (status = 500, description = "Internal server error")
    ),
    description = "Get CurseForge translation by mod ID",
    tag = "Translate",
)]
#[get("/curseforge/{mod_id}")]
async fn get_curseforge_translation(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let mod_id = path.into_inner();

    let service = CurseForgeService::new(data.db.clone());

    match service.get_translation(mod_id).await {
        Ok(translation) => Ok(web::Json(translation)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    post,
    path = "/translate/modrinth",
    request_body = ModrinthTranslationRequest,
    responses(
        (status = 200, description = "Translations found", body = Vec<ModrinthTranslationResponse>),
        (status = 500, description = "Internal server error")
    ),
    description = "Get Modrinth translations in batch by project IDs",
    tag = "Translate",
)]
#[post("/modrinth")]
async fn get_modrinth_translation_batch(
    data: web::Data<AppState>,
    body: web::Json<ModrinthTranslationRequest>,
) -> Result<impl Responder, ApiError> {
    let project_ids = body.project_ids.clone();
    let service = ModrinthService::new(data.db.clone());

    match service.get_translations_batch(project_ids).await {
        Ok(translation) => Ok(web::Json(translation)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    post,
    path = "/translate/curseforge",
    request_body = CurseForgeTranslationRequest,
    responses(
        (status = 200, description = "Translations found", body = Vec<CurseForgeTranslationResponse>),
        (status = 500, description = "Internal server error")
    ),
    description = "Get CurseForge translations in batch by mod IDs",
    tag = "Translate",
)]
#[post("/curseforge")]
async fn get_curseforge_translation_batch(
    data: web::Data<AppState>,
    body: web::Json<CurseForgeTranslationRequest>,
) -> Result<impl Responder, ApiError> {
    let mod_ids = body.modids.clone();
    let service = CurseForgeService::new(data.db.clone());

    match service.get_translations_batch(mod_ids).await {
        Ok(translation) => Ok(web::Json(translation)),
        Err(e) => Err(ApiError::from(e)),
    }
}
