pub mod modrinth;
pub mod curseforge;
pub mod translate;
pub mod common;

pub use modrinth::*;
pub use curseforge::*;
pub use translate::*;
pub use common::*;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(translate::config)
         .configure(common::config)
         .configure(translate::config)
         .configure(curseforge::config)
         .configure(modrinth::config);
}