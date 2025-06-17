use actix_web::{get, post, web, Responder};


use crate::config::AppState;
use crate::errors::{ApiError};
use crate::models::modrinth::entities::*;
use crate::models::modrinth::requests::*;
use crate::models::modrinth::responses::*;
use crate::services::modrinth::ModrinthService;
use crate::utils::redis_cache::{cacheable_json, create_key};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/modrinth/v2")
            .service(search_cached)
            .service(get_project)
            .service(get_projects)
            .service(get_project_versions)
            .service(get_version)
            .service(get_versions)
            .service(get_version_file)
            .service(get_version_files)
            .service(update_version_file)
            .service(update_version_files)
            .service(get_categories)
            .service(get_loaders)
            .service(get_game_versions),
    );
}

#[get("/")]
pub async fn root() -> impl Responder {
    "Modrinth API"
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/search",
    params(
        ("query" = Option<String>, Query, description = "Search query", example = "sodium"),
        ("facets" = Option<String>, Query, description = "Facets to filter results", example = "[['categories:forge'],['versions:1.17.1'],['project_type:mod'],['license:mit']]"),
        ("offset" = Option<i32>, Query, description = "Offset for pagination", example = "0"),
        ("limit" = Option<i32>, Query, description = "Limit for pagination", example = "10"),
        ("index" = Option<String>, Query, description = "Index to sort", example = "relevance")
    ),
    responses(
        (status = 200, description = "Search results found", body = SearchResponse)
    ),
    tag = "Modrinth"
)]
#[get("/search")]
pub async fn search_cached(
    web::Query(query): web::Query<SearchQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let redis_pool = data.redis_pool.clone();
    let db = data.db.clone();
    let modrinth_api_url = data.modrinth_api_url.clone();

    let key = create_key(
        "GET".to_string(),
        "/modrinth/search".to_string(),
        query.to_string(),
    );

    cacheable_json(
        redis_pool,
        key,
        3600, // 缓存 1 小时
        move || {
            Box::pin(async move {
                let service = ModrinthService::new(db);
                service
                    .search(
                        query.query,
                        query.facets,
                        query.offset,
                        query.limit,
                        query.index,
                        &modrinth_api_url,
                    )
                    .await
                    .map_err(|e| e.into())
            })
        },
    )
    .await
}

// pub async fn search(
//     web::Query(query): web::Query<SearchQuery>,
//     data: web::Data<AppState>,
// ) -> Result<impl Responder, ApiError> {
//     let service = ModrinthService::new(data.db.clone());
//     match service.search(
//         query.query,
//         query.facets,
//         query.offset,
//         query.limit,
//         query.index,
//         &data.modrinth_api_url,
//     ).await {
//         Ok(response) => Ok(web::Json(response)),
//         Err(e) => Err(ApiError::from(e)),
//     }
// }

#[utoipa::path(
    get,
    path = "/modrinth/v2/project/{project_id}",
    params(
        ("project_id" = String, Query, description = "ID of the game to filter project"),
    ),
    responses(
        (status = 200, description = "Project found", body = Project)
    ),
    tag = "Modrinth"
)]
#[get("/project/{project_id}")]
pub async fn get_project(
    idslug: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service.get_project_by_id_or_slug(idslug.into_inner()).await {
        Ok(project) => Ok(web::Json(project)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/projects",
    params(
        ("ids" = String, Query, description = "The IDs and/or slugs of the projects", example = "['AABBCCDD', 'EEFFGGHH]'")
    ),
    responses(
        (status = 200, description = "Projects Found", body = Vec<Project>),
    ),
    tag = "Modrinth"
)]
#[get("/projects")]
pub async fn get_projects(
    web::Query(project_ids): web::Query<ProjectIds>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let ids: Vec<&str> = serde_json::from_str(&project_ids.ids)
        .map_err(|_| ApiError::BadRequest("Invalid JSON format for ids".to_string()))?;

    let service = ModrinthService::new(data.db.clone());
    match service
        .get_projects(ids.into_iter().map(|s| s.to_string()).collect())
        .await
    {
        Ok(projects) => Ok(web::Json(projects)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/project/{project_id}/version",
    params(
        ("project_id" = String, Path, description = "ID or slug of the project")
    ),
    responses(
        (status = 200, description = "Project versions found", body = Vec<Version>)
    ),
    tag = "Modrinth"
)]
#[get("/project/{project_id}/version")]
pub async fn get_project_versions(
    idslug: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service.get_project_all_versions(idslug.into_inner()).await {
        Ok(versions) => Ok(web::Json(versions)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/version/{version_id}",
    params(
        ("version_id" = String, Path, description = "ID of the version")
    ),
    responses(
        (status = 200, description = "Version found", body = Version)
    ),
    tag = "Modrinth"
)]
#[get("/version/{version_id}")]
pub async fn get_version(
    version_id: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service.get_version(version_id.into_inner()).await {
        Ok(version) => Ok(web::Json(version)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/versions",
    params(
        ("ids" = String, Query, description = "The IDs of the versions", example = "['AABBCCDD', 'EEFFGGHH]'")
    ),
    responses(
        (status = 200, description = "Versions found", body = Vec<Version>)
    ),
    tag = "Modrinth"
)]
#[get("/versions")]
pub async fn get_versions(
    web::Query(version_ids): web::Query<ProjectIds>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let ids: Vec<&str> = serde_json::from_str(&version_ids.ids)
        .map_err(|_| ApiError::BadRequest("Invalid JSON format for ids".to_string()))?;

    let service = ModrinthService::new(data.db.clone());
    match service
        .get_versions(ids.into_iter().map(|s| s.to_string()).collect())
        .await
    {
        Ok(versions) => Ok(web::Json(versions)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/version_file/{hash}",
    params(
        ("hash" = String, Path, description = "Hash of the file, sha1 or sha512")
    ),
    responses(
        (status = 200, description = "File found", body = Version)
    ),
    tag = "Modrinth"
)]
#[get("/version_file/{hash}")]
pub async fn get_version_file(
    hash: web::Path<String>,
    query: web::Query<AlgorithmItems>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service
        .get_version_file(hash.into_inner(), query.algorithm.clone())
        .await
    {
        Ok(version) => Ok(web::Json(version)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    post,
    path = "/modrinth/v2/version_files",
    request_body = HashesQuery,
    responses(
        (status = 200, description = "Files found", body = std::collections::HashMap<String, Version>)
    ),
    tag = "Modrinth"
)]
#[post("/version_files")]
pub async fn get_version_files(
    body: web::Json<HashesQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service
        .get_version_files(body.hashes.clone(), body.algorithm.clone())
        .await
    {
        Ok(versions) => Ok(web::Json(versions)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    post,
    path = "/modrinth/v2/version_file/{hash}/update",
    request_body = UpdateItems,
    params(
        ("hash" = String, Path, description = "Hash of the file, sha1 or sha512"),
        ("algorithm" = String, Query, description = "Hash algorithm used, sha1 or sha512")
    ),
    responses(
        (status = 200, description = "File updated", body = Version)
    ),
    tag = "Modrinth"
)]
#[post("/version_file/{hash}/update")]
pub async fn update_version_file(
    hash: web::Path<String>,
    body: web::Json<UpdateItems>,
    query: web::Query<AlgorithmItems>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service
        .get_version_file_update(
            hash.into_inner(),
            query.algorithm.clone(),
            body.loaders.clone(),
            body.game_versions.clone(),
        )
        .await
    {
        Ok(version) => Ok(web::Json(version)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    post,
    path = "/modrinth/v2/version_files/update",
    responses(
        (status = 200, description = "Versions Found", body = Vec<Version>)
    ),
    tag = "Modrinth"
)]
#[post("/version_files/update")]
pub async fn update_version_files(
    body: web::Json<MultiUpdateItems>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service
        .get_version_files_update(body.hashes.clone(), body.algorithm.clone(), body.loaders.clone(), body.game_versions.clone())
        .await
    {
        Ok(versions) => Ok(web::Json(versions)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/tag/category",
    responses(
        (status = 200, description = "Categories found", body = Vec<Category>)
    ),
    tag = "Modrinth"
)]
#[get("/tag/category")]
pub async fn get_categories(data: web::Data<AppState>) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service.get_categories().await {
        Ok(categories) => Ok(web::Json(categories)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/tag/loader",
    responses(
        (status = 200, description = "Loaders found", body = Vec<Loader>)
    ),
    tag = "Modrinth"
)]
#[get("/tag/loader")]
pub async fn get_loaders(data: web::Data<AppState>) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service.get_loaders().await {
        Ok(loaders) => Ok(web::Json(loaders)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[utoipa::path(
    get,
    path = "/modrinth/v2/tag/game_version",
    responses(
        (status = 200, description = "Game versions found", body = Vec<GameVersion>)
    ),
    tag = "Modrinth"
)]
#[get("/tag/game_version")]
pub async fn get_game_versions(data: web::Data<AppState>) -> Result<impl Responder, ApiError> {
    let service = ModrinthService::new(data.db.clone());
    match service.get_game_versions().await {
        Ok(game_versions) => Ok(web::Json(game_versions)),
        Err(e) => Err(ApiError::from(e)),
    }
}