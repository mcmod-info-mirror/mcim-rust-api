pub mod common;
pub mod curseforge;
pub mod modrinth;
pub mod translate;

use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
