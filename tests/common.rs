use actix_web::{
    body::to_bytes,
    dev::Service,
    test::{TestRequest, init_service},
};

use mcim_rust_api::models::common::responses::StatisticsResponse;
use mcim_rust_api::test_utils::create_test_app;

#[actix_web::test]
async fn test_root() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get().uri("/").to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );
}

#[actix_web::test]
async fn test_statistics() {
    let app = init_service(create_test_app().await).await;

    let req = TestRequest::get().uri("/statistics").to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );
}

#[actix_web::test]
async fn test_translation_statistics_filters_null_translated_at() {
    let app = init_service(create_test_app().await).await;

    // Request statistics with translate=true
    let req = TestRequest::get()
        .uri("/statistics?translate=true")
        .to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success status, got: {}",
        response.status()
    );

    // Parse the response body
    let body = to_bytes(response.into_body()).await.unwrap();
    let stats: StatisticsResponse =
        serde_json::from_slice(&body).expect("Failed to parse response");

    // The translation statistics should exist
    assert!(
        stats.translate.is_some(),
        "Translation statistics should be present"
    );

    let translate_stats = stats.translate.unwrap();

    // Verify that the counts are present
    if let Some(&modrinth_count) = translate_stats.get("modrinth") {
        // With the test data, we have 2 entries with translated_at and 2 without
        // The count should be 2 (only those with non-null translated_at)
        println!("Modrinth translation count: {}", modrinth_count);
        assert_eq!(
            modrinth_count, 2,
            "Expected modrinth translation count to be 2 (only entries with non-null translated_at)"
        );
    }

    if let Some(&curseforge_count) = translate_stats.get("curseforge") {
        // With the test data, we have 2 entries with translated_at and 1 without
        // The count should be 2 (only those with non-null translated_at)
        println!("CurseForge translation count: {}", curseforge_count);
        assert_eq!(
            curseforge_count, 2,
            "Expected curseforge translation count to be 2 (only entries with non-null translated_at)"
        );
    }
}
