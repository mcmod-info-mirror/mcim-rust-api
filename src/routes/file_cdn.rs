use actix_web::{Responder, route, web, web::Redirect};

use crate::utils::app::AppState;
use crate::utils::file_cdn_load_balance::select_cdn_endpoint;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_modrinth_file)
        .service(get_curseforge_file)
        .service(get_curseforge_avatar)
        .service(get_curseforge_avatar_thumbnail)
        .service(get_modrinth_avatar);
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
    
    if data.file_cdn_enabled == false {
        let url = format!(
            "{}/files/{}/{}/{}",
            data.curseforge_file_cdn_fallback_url.clone(), file_id1, file_id2, file_name
        );
        return Redirect::to(url).temporary();
    }

    let mirror_url = select_cdn_endpoint(
        data.curseforge_file_cdn_url.clone(),
        data.curseforge_file_cdn_fallback_url.clone(),
        data.curseforge_cdn_primary_percentage,
    );
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
    if data.file_cdn_enabled == false {
        let url = format!(
            "{}/data/{}/versions/{}/{}",
            data.modrinth_file_cdn_fallback_url.clone(), project_id, version_id, file_name
        );
        return Redirect::to(url).temporary();
    }

    let mirror_url = select_cdn_endpoint(
        data.modrinth_file_cdn_url.clone(),
        data.modrinth_file_cdn_fallback_url.clone(),
        data.modrinth_cdn_primary_percentage,
    );

    let url = format!(
        "{}/data/{}/versions/{}/{}",
        mirror_url, project_id, version_id, file_name
    );

    // Redirect to the constructed URL
    Redirect::to(url).temporary()
}

#[utoipa::path(
    get,
    path = "/avatars/{modid1}/{modid2}/{file_name}",
    params(
        ("modid1" = String, Path, description = "ModId segment 1 of avatar path"),
        ("modid2" = String, Path, description = "ModId segment 2 of avatar path"),
        ("file_name" = String, Path, description = "Avatar file name")
    ),
    responses(
        (status = 301, description = "Curseforge Avatar Redirect"),
        (status = 500, description = "Internal server error")
    ),
    description = "Curseforge Avatar CDN endpoint (flat)",
    tag = "File CDN"
)]
#[route(
    "/avatars/{modid1}/{modid2}/{file_name}",
    method = "GET",
    method = "HEAD"
)]
pub async fn get_curseforge_avatar(
    path: web::Path<(String, String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (modid1, modid2, file_name) = path.into_inner();
    let avatar_path = format!("{}/{}/{}", modid1, modid2, file_name);
    if data.file_cdn_enabled == false {
        let url = format!(
            "{}/avatars/{}",
            data.curseforge_avatar_cdn_fallback_url.clone(), avatar_path
        );
        return Redirect::to(url).temporary();
    }

    let mirror_url = select_cdn_endpoint(
        data.curseforge_avatar_cdn_url.clone(),
        data.curseforge_avatar_cdn_fallback_url.clone(),
        data.curseforge_avatar_cdn_primary_percentage,
    );

    let url = format!("{}/avatars/{}", mirror_url, avatar_path);
    Redirect::to(url).temporary()
}

#[utoipa::path(
    get,
    path = "/avatars/thumbnails/{modid1}/{modid2}/{w}/{h}/{file_name}",
    params(
        ("modid1" = String, Path, description = "ModId segment 1"),
        ("modid2" = String, Path, description = "ModId segment 2"),
        ("w" = String, Path, description = "Thumbnail width"),
        ("h" = String, Path, description = "Thumbnail height"),
        ("file_name" = String, Path, description = "Thumbnail file name")
    ),
    responses(
        (status = 301, description = "Curseforge Avatar Thumbnail Redirect"),
        (status = 500, description = "Internal server error")
    ),
    description = "Curseforge Avatar CDN endpoint (thumbnails)",
    tag = "File CDN"
)]
#[route(
    "/avatars/thumbnails/{modid1}/{modid2}/{w}/{h}/{file_name}",
    method = "GET",
    method = "HEAD"
)]
pub async fn get_curseforge_avatar_thumbnail(
    path: web::Path<(String, String, String, String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (modid1, modid2, w, h, file_name) = path.into_inner();
    let avatar_path = format!("thumbnails/{}/{}/{}/{}/{}", modid1, modid2, w, h, file_name);
    if data.file_cdn_enabled == false {
        let url = format!(
            "{}/avatars/{}",
            data.curseforge_avatar_cdn_fallback_url.clone(), avatar_path
        );
        return Redirect::to(url).temporary();
    }

    let mirror_url = select_cdn_endpoint(
        data.curseforge_avatar_cdn_url.clone(),
        data.curseforge_avatar_cdn_fallback_url.clone(),
        data.curseforge_avatar_cdn_primary_percentage,
    );

    let url = format!("{}/avatars/{}", mirror_url, avatar_path);
    Redirect::to(url).temporary()
}

#[utoipa::path(
    get,
    path = "/data/{project_id}/{file_name}",
    params(
        ("project_id" = String, Path, description = "Project ID of the Modrinth project"),
        ("file_name" = String, Path, description = "Icon file name")
    ),
    responses(
        (status = 301, description = "Modrinth Icon Redirect"),
        (status = 500, description = "Internal server error")
    ),
    description = "Modrinth Icon CDN",
    tag = "File CDN"
)]
#[route(
    "/data/{project_id}/{file_name}",
    method = "GET",
    method = "HEAD"
)]
pub async fn get_modrinth_avatar(
    path: web::Path<(String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (project_id, file_name) = path.into_inner();
    if data.file_cdn_enabled == false {
        let url = format!(
            "{}/data/{}/{}",
            data.modrinth_avatar_cdn_fallback_url.clone(), project_id, file_name
        );
        return Redirect::to(url).temporary();
    }

    let mirror_url = select_cdn_endpoint(
        data.modrinth_avatar_cdn_url.clone(),
        data.modrinth_avatar_cdn_fallback_url.clone(),
        data.modrinth_avatar_cdn_primary_percentage,
    );

    let url = format!("{}/data/{}/{}", mirror_url, project_id, file_name);
    Redirect::to(url).temporary()
}
