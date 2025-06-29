use actix_web::{Responder, HttpResponse};
use utoipa::OpenApi;
use utoipauto::utoipauto;

#[allow(unknown_lints)]
#[utoipauto]
#[derive(OpenApi)]
#[openapi(info(
    title = "MCIM API",
    version = "1.0.0",
    contact(
        name = "mcmod-info-mirror",
        url = "https://github.com/mcmod-info-mirror"
    )
))]
pub struct OpenApiDoc;

pub async fn serve_openapi() -> impl Responder {
    let openapi_string = OpenApiDoc::openapi()
        .to_json()
        .expect("Should serialize to JSON");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(openapi_string.to_string())
}