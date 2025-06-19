use actix_web::{
    dev::Service,
    test::{init_service, TestRequest},
};

use mcim_rust_api::test_utils::create_test_app;

const MOD_IDS: [i32; 2] = [238222, 1004027];

const PROJECT_IDS: &[&str] = &["Wnxd13zP", "Ua7DFN59"];

#[actix_web::test]
async fn test_curseforge_single_translate() {
    let app = init_service(create_test_app().await).await;

    for modid in MOD_IDS {
        let req = TestRequest::get()
            .uri(&format!("/translate/curseforge/{}", modid))
            .to_request();
        let response = app.call(req).await.unwrap();
        assert!(
            response.status().is_success(),
            "Expected success status for modid {}, got: {}",
            modid,
            response.status()
        );
    }
}

#[actix_web::test]
async fn test_curseforge_translate_batch() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::post()
        .uri("/translate/curseforge")
        .set_json(&serde_json::json!({ "modids": MOD_IDS }))
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
    for project_id in PROJECT_IDS {
        let req = TestRequest::get()
            .uri(&format!("/translate/modrinth/{}", project_id))
            .to_request();
        let response = app.call(req).await.unwrap();
        assert!(
            response.status().is_success(),
            "Expected success status for project_id {}, got: {}",
            project_id,
            response.status()
        );
    }
}

#[actix_web::test]
async fn test_modrinth_translate_batch() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::post()
        .uri(&format!("/translate/modrinth"))
        .set_json(&serde_json::json!({ "project_ids": PROJECT_IDS }))
        .to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );
}
