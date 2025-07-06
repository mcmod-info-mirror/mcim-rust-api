use actix_web::{get, post, web, Responder};

use crate::config::AppState;
use crate::errors::{ApiError, ServiceError};
use crate::models::curseforge::requests::*;
use crate::models::curseforge::responses::*;
use crate::services::curseforge::CurseforgeService;
use crate::utils::redis_cache::{cacheable_json, create_key};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/curseforge").service(root).service(
            web::scope("/v1")
                .service(search_mods_cached)
                .service(get_categories)
                .service(get_mod)
                .service(get_mods)
                .service(get_file)
                .service(get_file_download_url)
                .service(get_mod_files)
                .service(get_files_by_ids)
                .service(get_fingerprints)
                .service(get_fingerprints_by_game_id)
        ),
    );
}

#[utoipa::path(
    get,
    path = "/curseforge",
    responses(
        (status = 200, description = "Curseforge Message"),
    ),
    description = "Root endpoint for the Curseforge API",
    tag = "Curseforge",
    operation_id = "curseforge_root"
)]
#[get("/")]
pub async fn root() -> impl Responder {
    "Curseforge API"
}

#[utoipa::path(
    get,
    path = "/curseforge/v1/mods/search",
    params(
        ("gameId" = i32, Query, description = "ID of the game to filter mods by (optional)"),
        ("classId" = Option<i32>, Query, description = "ID of the class to filter mods by (optional)"),
        ("categoryId" = Option<i32>, Query, description = "ID of the category to filter mods by (optional)"),
        ("categoryIds" = Option<String>, Query, description = "Comma-separated list of category IDs to filter mods by (optional)"),
        ("gameVersion" = Option<String>, Query, description = "Game version filter (optional)"),
        ("gameVersions" = Option<String>, Query, description = "Comma-separated list of game versions to filter mods by (optional)"),
        ("searchFilter" = Option<String>, Query, description = "Search filter for mod names (optional)"),
        ("sortField" = Option<String>, Query, description = "Field to sort results by (optional)"),
        ("sortOrder" = Option<String>, Query, description = "Order to sort results in (asc/desc) (optional)"),
        ("modLoaderType" = Option<String>, Query, description = "Mod loader type filter (optional)"),
        ("modLoaderTypes" = Option<String>, Query, description = "Comma-separated list of mod loader types to filter mods by (optional)"),
        ("gameVersionTypeId" = Option<i32>, Query, description = "Game version type ID filter (optional)"),
        ("authorId" = Option<i32>, Query, description = "ID of the author to filter mods by (optional)"),
        ("primaryAuthorId" = Option<i32>, Query, description = "ID of the primary author to filter mods by (optional)"),
        ("slug" = Option<String>, Query, description = "Slug of the mod to retrieve (optional)"),
        ("index" = Option<i32>, Query, description = "Zero-based index of the first item to include in the response. The limit is: (index + pageSize <= 10,000).", example = 0, maximum = 10000, minimum = 0),
        ("pageSize" = Option<i32>, Query, description = "Number of items to include in the response. The default/maximum value is 50.", example = 50, maximum = 50, minimum = 1)
    ),
    responses(
        (status = 200, description = "Search Result found", body = SearchResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Search"
)]
#[get("/mods/search")]
async fn search_mods_cached(
    query: web::Query<SearchQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let redis_pool = data.redis_pool.clone();
    let db = data.db.clone();
    let curseforge_api_url = data.curseforge_api_url.clone();
    let curseforge_api_key = data.curseforge_api_key.clone();

    let key = create_key(
        "GET".to_string(),
        "/curseforge/v1/mods/search".to_string(),
        query.to_string(),
    );

    cacheable_json(
        redis_pool.clone(),
        key,
        3600, // 缓存 1 小时
        move || {
            let service = CurseforgeService::new(db.clone(), redis_pool);
            Box::pin(async move {
                service
                    .search_mods(&query, &curseforge_api_url, &curseforge_api_key)
                    .await
                    .map_err(Into::into)
            })
        },
    )
    .await
}

// async fn search_mods(
//     query: web::Query<SearchQuery>,
//     data: web::Data<AppState>,
// ) -> Result<impl Responder, ApiError> {
//     let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

//     match service.search_mods(
//         &query,
//         &data.curseforge_api_url,
//         &data.curseforge_api_key,
//     ).await {
//         Ok(search_result) => Ok(web::Json(search_result)),
//         Err(e) => Err(e.into()),
//     }
// }

#[utoipa::path(
    get,
    path = "/curseforge/v1/mods/{mod_id}",
    params(
        ("mod_id" = i32, Path, description = "ID of the mod to retrieve")
    ),
    responses(
        (status = 200, description = "Mod found", body = ModResponse),
        (status = 404, description = "Mod not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get Mod by ModId"
)]
#[get("/mods/{mod_id}")]
async fn get_mod(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let mod_id = path.into_inner();

    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service.get_mod(mod_id).await {
        Ok(Some(mod_data)) => Ok(web::Json(mod_data)),
        Ok(None) => Err(ServiceError::NotFound {
            resource: "Mod".to_string(),
            detail: Some(format!("Mod with ID {} not found", mod_id)),
        }
        .into()),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    post,
    path = "/curseforge/v1/mods",
    request_body = ModsBody,
    responses(
        (status = 200, description = "Mods found", body = Vec<ModResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get Mods by ModIds"
)]
#[post("/mods")]
async fn get_mods(
    body: web::Json<ModsBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service.get_mods(body.mod_ids.clone()).await {
        Ok(mods) => Ok(web::Json(mods)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    get,
    path = "/curseforge/v1/mods/{mod_id}/files",
    params(
        ("mod_id" = i32, Path, description = "ID of the mod to retrieve files for"),
        ("gameVersion" = Option<String>, Query, description = "Game version filter (optional)"),
        ("modLoaderType" = Option<String>, Query, description = "Mod loader type filter (optional)"),
        ("index" = Option<i32>, Query, description = "Index for pagination (optional)"),
        ("pageSize" = Option<i32>, Query, description = "Page size for pagination (optional)")
    ),
    responses(
        (status = 200, description = "Files found", body = Vec<FileResponse>),
        (status = 404, description = "Mod not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get Mod Files by ModId"
)]
#[get("/mods/{mod_id}/files")]
async fn get_mod_files(
    path: web::Path<i32>,
    query: web::Query<ModFilesQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let mod_id = path.into_inner();
    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service
        .get_mod_files(
            mod_id,
            query.game_version.clone(),
            query.mod_loader_type,
            query.index,
            query.page_size,
        )
        .await
    {
        Ok(files) => Ok(web::Json(files)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    get,
    path = "/curseforge/v1/mods/{mod_id}/files/{file_id}/download-url",
    params(
        ("mod_id" = i32, Path, description = "ID of the mod"),
        ("file_id" = i32, Path, description = "ID of the file to retrieve download URL for")
    ),
    responses(
        (status = 200, description = "Download URL found", body = DownloadUrlResponse),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get File Download URL by ModId and FileId"
)]
#[get("/mods/{mod_id}/files/{file_id}/download-url")]
async fn get_file_download_url(
    path: web::Path<(i32, i32)>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let (mod_id, file_id) = path.into_inner();

    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service.get_file_download_url(mod_id, file_id).await {
        Ok(url) => Ok(web::Json(url)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    get,
    path = "/curseforge/v1/mods/{mod_id}/files/{file_id}",
    params(
        ("mod_id" = i32, Path, description = "ID of the mod to which the file belongs"),
        ("file_id" = i32, Path, description = "ID of the file to retrieve")
    ),
    responses(
        (status = 200, description = "File found", body = FileResponse),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get File by ModId and FileId"
)]
#[get("/mods/{mod_id}/files/{file_id}")]
async fn get_file(
    path: web::Path<(i32, i32)>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let (_, file_id) = path.into_inner();

    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service.get_file(file_id).await {
        Ok(file_data) => Ok(web::Json(file_data)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    post,
    path = "/curseforge/v1/mods/files",
    request_body = FileIdsBody,
    responses(
        (status = 200, description = "Files found", body = Vec<FileResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get Files by FileIds"
)]
#[post("/mods/files")]
async fn get_files_by_ids(
    body: web::Json<FileIdsBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service.get_files(body.file_ids.clone()).await {
        Ok(files) => Ok(web::Json(files)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    post,
    path = "/curseforge/v1/fingerprints",
    request_body = FingerprintsBody,
    responses(
        (status = 200, description = "Fingerprints found", body = FingerprintResult),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get Fingerprints by Fingerprints"
)]
#[post("/fingerprints")]
async fn get_fingerprints(
    body: web::Json<FingerprintsBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service
        .get_fingerprints(body.fingerprints.clone(), None)
        .await
    {
        Ok(fingerprint_result) => Ok(web::Json(fingerprint_result)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    post,
    path = "/curseforge/v1/fingerprints/{game_id}",
    params(
        ("game_id" = i32, Path, description = "ID of the game to filter fingerprints by")
    ),
    request_body = FingerprintsBody,
    responses(
        (status = 200, description = "Fingerprints found", body = FingerprintResult),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get Fingerprints by GameId and Fingerprints"
)]
#[post("/fingerprints/{game_id}")]
async fn get_fingerprints_by_game_id(
    path: web::Path<i32>,
    body: web::Json<FingerprintsBody>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let game_id = path.into_inner();

    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service
        .get_fingerprints(body.fingerprints.clone(), Some(game_id))
        .await
    {
        Ok(fingerprint_result) => Ok(web::Json(fingerprint_result)),
        Err(e) => Err(e.into()),
    }
}

#[utoipa::path(
    get,
    path = "/curseforge/v1/categories",
    params(
        ("gameId" = i32, Query, description = "ID of the game to filter categories by"),
        ("classId" = Option<i32>, Query, description = "ID of the class to filter categories by (optional)"),
        ("classOnly" = Option<bool>, Query, description = "Whether to return only classes (optional)")
    ),
    responses(
        (status = 200, description = "Categories found", body = CategoriesResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Curseforge",
    description = "Curseforge Get Categories by GameId and ClassId"
)]
#[get("/categories")]
async fn get_categories(
    query: web::Query<CategoriesQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = CurseforgeService::new(data.db.clone(), data.redis_pool.clone());

    match service
        .get_categories(query.game_id, query.class_id, query.class_only)
        .await
    {
        Ok(categories) => Ok(web::Json(categories)),
        Err(e) => Err(e.into()),
    }
}
