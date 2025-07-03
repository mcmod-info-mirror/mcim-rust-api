use actix_web::{route, web, web::Redirect, Responder};

use crate::config::AppState;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_curseforge_file).service(get_modrinth_file);
}

#[utoipa::path(
    get,
    path = "/files/{file_id1}/{file_id2}/{file_name}",
    params(
        ("file_id1" = i32, Path, description = "First part of the Curseforge file ID"),
        ("file_id2" = i32, Path, description = "Second part of the Curseforge file ID"),
        ("file_name" = String, Path, description = "Name of the file to be downloaded")
    ),
    responses(
        (status = 301, description = "Curseforge File Redirect"),
        (status = 500, description = "Internal server error")
    ),
    description = "Curseforge File CDN endpoint",
    tag = "File CDN"
)]
#[route(
    "/files/{file_id1}/{file_id2}/{file_name}",
    method = "GET",
    method = "HEAD"
)]
pub async fn get_curseforge_file(
    path: web::Path<(String, String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (file_id1, file_id2, file_name) = path.into_inner();
    let mirror_url = data.curseforge_file_cdn_url.clone();
    let encoded_file_name = urlencoding::encode(&file_name).to_string();
    let url = format!(
        "{}/files/{}/{}/{}",
        mirror_url, file_id1, file_id2, encoded_file_name
    );

    // Redirect to the constructed URL
    Redirect::to(url).temporary()
}

#[utoipa::path(
    get,
    path = "/data/{project_id}/versions/{version_id}/{file_name}",
    params(
        ("project_id" = String, Path, description = "Project ID of the Modrinth project"),
        ("version_id" = String, Path, description = "Version ID of the Modrinth project"),
        ("file_name" = String, Path, description = "Name of the file to be downloaded")
    ),
    responses(
        (status = 301, description = "Modrinth File Redirect"),
        (status = 500, description = "Internal server error")
    ),
    description = "Modrinth File CDN endpoint",
    tag = "File CDN"
)]
#[route(
    "/data/{project_id}/versions/{version_id}/{file_name}",
    method = "GET",
    method = "HEAD"
)]
pub async fn get_modrinth_file(
    path: web::Path<(String, String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (project_id, version_id, file_name) = path.into_inner();
    let mirror_url = data.modrinth_file_cdn_url.clone();

    let url = format!(
        "{}/data/{}/versions/{}/{}",
        mirror_url, project_id, version_id, file_name
    );

    // Redirect to the constructed URL
    Redirect::to(url).temporary()
}
