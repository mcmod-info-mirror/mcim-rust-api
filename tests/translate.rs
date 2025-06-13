use actix_web::{
    dev::Service,
    test::{init_service, TestRequest},
};

use mcim_rust_api::test_utils::create_test_app;

#[actix_web::test]
async fn test_curseforge_single_translate() {
    let app = init_service(create_test_app().await).await;
    let modid = "238222";

    let req = TestRequest::get()
        .uri(&format!("/translate/curseforge/{}", modid))
        .to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );
}

#[actix_web::test]
async fn test_curseforge_translate_batch() {
    let app = init_service(create_test_app().await).await;
    let modids = [238222, 1004027];
    let req = TestRequest::post()
        .uri(&format!("/translate/curseforge"))
        .set_json(&serde_json::json!({ "modids": modids }))
        .to_request();
    let response = app.call(req).await.unwrap();

    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );
}

#[actix_web::test]
async fn test_modrinth_single_translate() {
    let app = init_service(create_test_app().await).await;
    let project_id = "AANobbMI";

    let req = TestRequest::get()
        .uri(&format!("/translate/modrinth/{}", project_id))
        .to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );
}

#[actix_web::test]
async fn test_modrinth_translate_batch() {
    let app = init_service(create_test_app().await).await;
    let project_ids = ["AANobbMI", "P7dR8mSH"];
    let req = TestRequest::post()
        .uri(&format!("/translate/modrinth"))
        .set_json(&serde_json::json!({ "project_ids": project_ids }))
        .to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );
}