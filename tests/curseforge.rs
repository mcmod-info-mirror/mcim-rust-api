use actix_web::{
    body::to_bytes,
    dev::Service,
    test::{TestRequest, init_service},
};
use serde_json::json;

use mcim_rust_api::test_utils::create_test_app;

static MOD_ID: i32 = 594678;
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
        .uri("/curseforge/v1/mods/search?gameId=432&classId=6&index=0&pageSize=50")
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_mod_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/mods/{}", MOD_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
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
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
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
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
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
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_mod_files_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!("/curseforge/v1/mods/{}/files", MOD_ID))
        .to_request();

    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_mod_files_with_filters() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!(
            "/curseforge/v1/mods/{}/files?gameVersion=1.16.5&modLoaderType=4&index=0&pageSize=20",
            MOD_IDS[0]
        ))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_file_download_url() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!(
            "/curseforge/v1/mods/{}/files/{}/download-url",
            MOD_ID, FILE_ID
        ))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_fingerprints_success() {
    let app = init_service(create_test_app().await).await;

    let payload = json!({
        "fingerprints": FINGERPRINTS.clone()
    });

    let req = TestRequest::post()
        .uri("/curseforge/v1/fingerprints")
        .set_json(json!(payload))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_fingerprints_by_game_id_success() {
    let app = init_service(create_test_app().await).await;

    let payload = json!({
        "fingerprints": FINGERPRINTS.clone()
    });

    let req = TestRequest::post()
        .uri(&format!("/curseforge/v1/fingerprints/{}", GAME_ID))
        .set_json(json!(payload))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_not_found_fingerprint() {
    let app = init_service(create_test_app().await).await;
    let not_found_fingerprints: Vec<i64> = vec![11451419810, 1234567890];
    let payload = json!({
        "fingerprints": not_found_fingerprints
    });

    let req = TestRequest::post()
        .uri("/curseforge/v1/fingerprints")
        .set_json(json!(payload))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    let json_body =
        serde_json::from_str::<serde_json::Value>(&body).expect("Failed to parse JSON response");
    // body.data.unmatchedFingerprints == not_found_fingerprints
    let unmatched_fingerprints = json_body["data"]["unmatchedFingerprints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect::<Vec<i64>>();
    assert_eq!(
        unmatched_fingerprints, not_found_fingerprints,
        "Unmatched fingerprints do not match the expected values"
    );
}

#[actix_web::test]
async fn test_get_invalid_fingerprint() {
    let app = init_service(create_test_app().await).await;
    let not_found_fingerprints: Vec<i64> = vec![-2, -1];
    let payload = json!({
        "fingerprints": not_found_fingerprints
    });

    let req = TestRequest::post()
        .uri("/curseforge/v1/fingerprints")
        .set_json(json!(payload))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    let json_body =
        serde_json::from_str::<serde_json::Value>(&body).expect("Failed to parse JSON response");
    // body.data.unmatchedFingerprints == not_found_fingerprints
    let unmatched_fingerprints = json_body["data"]["unmatchedFingerprints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect::<Vec<i64>>();
    assert_eq!(
        unmatched_fingerprints, not_found_fingerprints,
        "Unmatched fingerprints do not match the expected values"
    );
}

#[actix_web::test]
async fn test_get_categories_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri("/curseforge/v1/categories?gameId=432")
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_categories_with_class_id_filter_success() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!(
            "/curseforge/v1/categories?gameId={}&classId={}",
            GAME_ID, CLASS_ID
        ))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_get_categories_with_class_only_filters() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get()
        .uri(&format!(
            "/curseforge/v1/categories?gameId={}&classesOnly=true",
            GAME_ID
        ))
        .to_request();

    let resp = app.call(req).await.unwrap();
    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}
