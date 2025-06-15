use actix_web::{
    dev::Service, test::{init_service, TestRequest}
};
use serde_json::json;

use mcim_rust_api::test_utils::create_test_app;

static MOD_ID : i32 = 594678;
static FILE_ID: i32 = 3913840;

static CLASS_ID: i32 = 6;
const GAME_ID: i32 = 432;
const MOD_IDS: [i32; 2] = [946010, 594678];
const FILE_IDS: [i32; 2] = [3913840, 5976953];
const FINGERPRINTS: [i32; 2] = [2070800629, 1904165976];


#[actix_web::test]
async fn test_search() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri("/curseforge/v1/mods/search?gameId=432&classId=6&index=0&pageSize=20")
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_mod_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/mods/{}", MOD_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_mod_not_found() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri("/curseforge/v1/mods/999999")
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_get_mod_invalid_id() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri("/curseforge/v1/mods/-1")
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_get_mods_success() {
    let app = init_service(create_test_app().await).await;

    let payload = json!({
        "modIds": MOD_IDS.clone(),
        "filterPcOnly": true
    });

    let req = TestRequest::post()
        .uri("/curseforge/v1/mods")
        .set_json(payload)
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_mods_not_found() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::post()
        .uri("/curseforge/v1/mods")
        .set_json(json!({
            "modIds": [1]
        }))
        .to_request();

    let resp = app.call(req).await.unwrap();

    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_get_file_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/mods/{}/files/{}", MOD_ID, FILE_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_files_success() {
    let app = init_service(create_test_app().await).await;

    let payload = json!({
        "fileIds": FILE_IDS.clone()
    });

    let req = TestRequest::post()
        .uri("/curseforge/v1/mods/files")
        .set_json(payload)
        .to_request();

    let resp = app.call(req).await.unwrap();
    eprint!("{:?}", resp);
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_mod_files_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/mods/{}/files", MOD_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_mod_files_with_filters() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/mods/{}/files?gameVersion=1.16.5&modLoaderType=4&index=0&pageSize=20", MOD_IDS[0]))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_file_download_url() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/mods/{}/files/{}/download-url", MOD_ID, FILE_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_fingerprints_success() {
    let app = init_service(create_test_app().await).await;

    let payload = json!({
        "fingerprints": FINGERPRINTS.clone()
    });

    let req = TestRequest::post()
        .uri("/curseforge/v1/fingerprints")
        .set_json(&json!(payload))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_fingerprints_by_game_id_success() {
    let app = init_service(create_test_app().await).await;

    let payload = json!({
        "fingerprints": FINGERPRINTS.clone()
    });

    let req = TestRequest::post()
        .uri(&format!("/curseforge/v1/fingerprints/{}", GAME_ID))
        .set_json(&json!(payload))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_categories_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri("/curseforge/v1/categories?gameId=432")
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_categories_with_class_id_filter_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/categories?gameId={}&classId={}", GAME_ID, CLASS_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_categories_with_class_only_filters() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/categories?gameId={}&classOnly=true", GAME_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();
    assert!(resp.status().is_success());
}
