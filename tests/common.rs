use actix_web::{
    body::to_bytes,
    dev::Service,
    test::{TestRequest, init_service},
};

use mcim_rust_api::models::common::responses::{
    FilePlatform, FileStatisticsResponse, StatisticsResponse,
};
use mcim_rust_api::services::file_stats::{
    curseforge_member, daily_key, modrinth_member, stats_today,
};
use mcim_rust_api::test_utils::{create_test_app, init_test_redis};
use redis::AsyncCommands;

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
        // With the test data, we have 2 entries, both with non-null translated_at
        println!("Modrinth translation count: {}", modrinth_count);
        assert_eq!(
            modrinth_count, 2,
            "Expected modrinth translation count to be 2 (only entries with non-null translated_at)"
        );
    }

    if let Some(&curseforge_count) = translate_stats.get("curseforge") {
        // With the test data, we have 2 entries, both with non-null translated_at
        println!("CurseForge translation count: {}", curseforge_count);
        assert_eq!(
            curseforge_count, 2,
            "Expected curseforge translation count to be 2 (only entries with non-null translated_at)"
        );
    }
}

#[actix_web::test]
async fn test_statistics_files_lists_only_files_above_threshold() {
    let cf_hot = curseforge_member("3913", "840", "hats-and-cosmetics-1.2.2-1.19.1.jar").unwrap();
    let mr_hot =
        modrinth_member("Wnxd13zP", "BBnb9L4J", "Clumps-neoforge-1.21-18.0.0.1.jar").unwrap();
    let cf_unknown = curseforge_member("9999", "1", "not in db+1.jar").unwrap();
    let at_threshold = curseforge_member("1234", "567", "at-threshold.jar").unwrap();

    let key = daily_key(stats_today());
    let mut redis = init_test_redis().await.as_ref().clone();
    let members = [
        (cf_hot.as_str(), 10_001),
        (mr_hot.as_str(), 20_000),
        (cf_unknown.as_str(), 15_000),
        (at_threshold.as_str(), 10_000),
    ];
    for (member, score) in members {
        let _: () = redis.zadd(&key, member, score).await.unwrap();
    }

    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get().uri("/statistics_files").to_request();
    let response = app.call(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "status: {}",
        response.status()
    );
    let body = to_bytes(response.into_body()).await.unwrap();
    let stats: FileStatisticsResponse = serde_json::from_slice(&body).unwrap();

    let _: () = redis
        .zrem(&key, members.map(|(member, _)| member).to_vec())
        .await
        .unwrap();

    let files: Vec<_> = stats
        .files
        .iter()
        .map(|f| (f.platform, f.url.as_str(), f.size, f.count))
        .collect();
    assert_eq!(
        files,
        vec![
            (
                FilePlatform::Modrinth,
                "https://cdn.modrinth.com/data/Wnxd13zP/versions/BBnb9L4J/Clumps-neoforge-1.21-18.0.0.1.jar",
                Some(18383),
                20_000,
            ),
            (
                FilePlatform::Curseforge,
                "https://edge.forgecdn.net/files/9999/1/not%20in%20db%2B1.jar",
                None,
                15_000,
            ),
            (
                FilePlatform::Curseforge,
                "https://edge.forgecdn.net/files/3913/840/hats-and-cosmetics-1.2.2-1.19.1.jar",
                Some(120660),
                10_001,
            ),
        ]
    );
}

#[actix_web::test]
async fn test_statistics_files_rejects_unretained_date() {
    let app = init_service(create_test_app().await).await;
    let date = stats_today() - chrono::Days::new(2);
    let req = TestRequest::get()
        .uri(&format!("/statistics_files?date={}", date))
        .to_request();
    let response = app.call(req).await.unwrap();
    assert_eq!(response.status().as_u16(), 400);
}
