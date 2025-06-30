pub mod common;
pub mod curseforge;
pub mod file_cdn;
pub mod modrinth;
pub mod openapi;
pub mod translate;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(translate::config)
        .configure(common::config)
        .configure(translate::config)
        .configure(curseforge::config)
        .configure(modrinth::config)
        .configure(file_cdn::config)
        .route("/openapi.json", web::get().to(openapi::serve_openapi));
}
