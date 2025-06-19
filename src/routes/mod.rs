pub mod modrinth;
pub mod curseforge;
pub mod translate;
pub mod common;
pub mod file_cdn;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(translate::config)
         .configure(common::config)
         .configure(translate::config)
         .configure(curseforge::config)
         .configure(modrinth::config)
         .configure(file_cdn::config);
}