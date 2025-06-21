use actix_web::{
    body::to_bytes,
    dev::Service,
    http::StatusCode,
    test::{init_service, TestRequest},
};
use serde_json::json;

use mcim_rust_api::test_utils::create_test_app;

// 测试数据常量
const PROJECT_IDS: &[&str] = &["Wnxd13zP", "Ua7DFN59"];
const SLUGS: &[&str] = &["sodium", "clumps"];
const VERSION_IDS: &[&str] = &[
    "dpSzBMP6", "IOIGqCVr", "xVBjqLw6", "8jfhokYb", "fTWVa6NX", "Km2A7nLe",
];
const SHA512_SAMPLE: &[&str] = &[
    "be134c430492bb8933ff60cc59ff6143b25c4b79aa0d4a6e0332d9de7cfe1bacd16a43fe17167e9cc57d4747237f68cf584b99dd78233799239fb6acc0807d61",
    "1c97698babd08c869f76c53e86b4cfca3f369d0fdf0d8237d5d37d03d37cc4de9fc6a831f00c5ce8de6b145e774a31d0adc301e85fb24a4649e9af5c75156a0f",
    "4962062a240a10d1eb3507b28477270d7557a2d3d83ef459f9939a4be32fa8f8fcc92c3eab5125b183f7da11a73cd9f06fb049a8b6cbc276fe3401bbede766de",
];
const SHA1_SAMPLE: &[&str] = &[
    "f0cea90219f681c3183e0d37d021cb8902d2d085",
    "627c93adb68e04ffb390ad0e5dbf62d342f27a28",
    "e8b77ed731002c41d0658d5386cfc25f0df12dc4",
    "d3bcef6c363422b38cbd0298af63a27b5e75829d",
];

struct UpdateSample {
    pub hash: String,
    pub algorithm: String,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
}

fn get_update_samples() -> Vec<UpdateSample> {
    vec![
        UpdateSample {
            hash: "f0cea90219f681c3183e0d37d021cb8902d2d085".to_string(),
            algorithm: "sha1".to_string(),
            loaders: vec!["fabric".to_string()],
            game_versions: vec!["1.20.5".to_string()],
        },
        UpdateSample {
            hash: "1c97698babd08c869f76c53e86b4cfca3f369d0fdf0d8237d5d37d03d37cc4de9fc6a831f00c5ce8de6b145e774a31d0adc301e85fb24a4649e9af5c75156a0f".to_string(),
            algorithm: "sha512".to_string(),
            loaders: vec!["forge".to_string()],
            game_versions: vec!["1.15.2".to_string()],
        },
    ]
}

#[actix_web::test]
async fn test_modrinth_root() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get().uri("/modrinth/").to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_search() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get()
        .uri("/modrinth/v2/search?query=sodium")
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_project_by_project_id() {
    let app = init_service(create_test_app().await).await;

    for project_id in PROJECT_IDS {
        let req = TestRequest::get()
            .uri(&format!("/modrinth/v2/project/{}", project_id))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_project_by_slug() {
    let app = init_service(create_test_app().await).await;

    for slug in SLUGS {
        let req = TestRequest::get()
            .uri(&format!("/modrinth/v2/project/{}", slug))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_projects_by_project_ids() {
    let app = init_service(create_test_app().await).await;
    let ids_json = serde_json::to_string(PROJECT_IDS).unwrap();
    let req = TestRequest::get()
        .uri(&format!(
            "/modrinth/v2/projects?ids={}",
            urlencoding::encode(&ids_json)
        ))
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_projects_by_slugs() {
    let app = init_service(create_test_app().await).await;
    let ids_json = serde_json::to_string(SLUGS).unwrap();
    let req = TestRequest::get()
        .uri(&format!(
            "/modrinth/v2/projects?ids={}",
            urlencoding::encode(&ids_json)
        ))
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_projects_empty() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get()
        .uri("/modrinth/v2/projects?ids=[]")
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    // need 200 ok, response body is []
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    assert_eq!(body, "[]", "Expected empty array, got: {}", body);
}

#[actix_web::test]
async fn test_modrinth_project_versions_by_project_id() {
    let app = init_service(create_test_app().await).await;

    for project_id in PROJECT_IDS {
        let req = TestRequest::get()
            .uri(&format!("/modrinth/v2/project/{}/version", project_id))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_project_versions_by_slug() {
    let app = init_service(create_test_app().await).await;

    for slug in SLUGS {
        let req = TestRequest::get()
            .uri(&format!("/modrinth/v2/project/{}/version", slug))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_version() {
    let app = init_service(create_test_app().await).await;

    for version_id in VERSION_IDS {
        let req = TestRequest::get()
            .uri(&format!("/modrinth/v2/version/{}", version_id))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_versions() {
    let app = init_service(create_test_app().await).await;
    let ids_json = serde_json::to_string(VERSION_IDS).unwrap();
    let req = TestRequest::get()
        .uri(&format!(
            "/modrinth/v2/versions?ids={}",
            urlencoding::encode(&ids_json)
        ))
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_versions_empty() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get()
        .uri("/modrinth/v2/versions?ids=[]")
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    assert_eq!(body, "[]", "Expected empty array, got: {}", body);
}

#[actix_web::test]
async fn test_modrinth_version_file_sha1() {
    let app = init_service(create_test_app().await).await;

    for sha1_hash in SHA1_SAMPLE {
        let req = TestRequest::get()
            .uri(&format!(
                "/modrinth/v2/version_file/{}?algorithm=sha1",
                sha1_hash
            ))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_version_file_sha512() {
    let app = init_service(create_test_app().await).await;

    for sha512_hash in SHA512_SAMPLE {
        let req = TestRequest::get()
            .uri(&format!(
                "/modrinth/v2/version_file/{}?algorithm=sha512",
                sha512_hash
            ))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_version_file_not_found() {
    let app = init_service(create_test_app().await).await;
    let algorithm_list = ["sha1", "sha512"];
    for algorithm in &algorithm_list {
        let req = TestRequest::get()
            .uri(&format!(
                "/modrinth/v2/version_file/qeq?algorithm={}",
                algorithm
            ))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(
            status == StatusCode::NOT_FOUND,
            "Expected 404 Not Found, got: {}, Body: {}",
            status,
            body
        );
    }
}

#[actix_web::test]
async fn test_modrinth_version_file_sha1_update() {
    let app = init_service(create_test_app().await).await;

    for sha1_hash in SHA1_SAMPLE {
        let req = TestRequest::post()
            .uri(&format!(
                "/modrinth/v2/version_file/{}/update?algorithm=sha1",
                sha1_hash
            ))
            .set_json(&json!({
                "loaders": ["fabric"],
                "game_versions": ["1.20.1"]
            }))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_version_file_sha512_update() {
    let app = init_service(create_test_app().await).await;

    for sample in get_update_samples() {
        if sample.algorithm != "sha512" {
            continue; // Skip samples that are not sha512
        }
        let req = TestRequest::post()
            .uri(&format!(
                "/modrinth/v2/version_file/{}/update?algorithm=sha512",
                sample.hash
            ))
            .set_json(&json!({
                "loaders": sample.loaders,
                "game_versions": sample.game_versions
            }))
            .to_request();
        let resp = app.call(req).await.unwrap();
        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_version_file_update_empty() {
    let app = init_service(create_test_app().await).await;

    let algorithm_list = ["sha1", "sha512"];
    for algorithm in algorithm_list {
        let req = TestRequest::post()
            .uri(&format!(
                "/modrinth/v2/version_file/qeq/update?algorithm={}",
                algorithm
            ))
            .set_json(&json!({
                "loaders": [],
                "game_versions": []
            }))
            .to_request();
        let resp = app.call(req).await.unwrap();
        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(
            status == StatusCode::NOT_FOUND,
            "Status: {}, Body: {}",
            status,
            body
        );
    }
}

#[actix_web::test]
async fn test_modrinth_version_files_sha1() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::post()
        .uri("/modrinth/v2/version_files")
        .set_json(&json!({
            "algorithm": "sha1",
            "hashes": SHA1_SAMPLE
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_version_files_sha512() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::post()
        .uri("/modrinth/v2/version_files")
        .set_json(&json!({
            "algorithm": "sha512",
            "hashes": SHA512_SAMPLE
        }))
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_version_files_empty() {
    let app = init_service(create_test_app().await).await;
    let algorithm_list = ["sha1", "sha512"];
    for algorithm in algorithm_list {
        let req = TestRequest::post()
            .uri("/modrinth/v2/version_files")
            .set_json(&json!({
                "hashes": [],
                "algorithm": algorithm
            }))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
        // need 200 ok, response body is {}
        assert_eq!(body, "{}", "Expected empty object, got: {}", body);
    }
}

#[actix_web::test]
async fn test_modrinth_version_files_sha1_update() {
    let app = init_service(create_test_app().await).await;
    for sample in get_update_samples() {
        if sample.algorithm != "sha1" {
            continue; // Skip samples that are not sha1
        }
        let req = TestRequest::post()
            .uri("/modrinth/v2/version_files/update")
            .set_json(&json!({
                "hashes": [sample.hash],
                "algorithm": sample.algorithm,
                "loaders": sample.loaders,
                "game_versions": sample.game_versions
            }))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}

#[actix_web::test]
async fn test_modrinth_version_files_sha512_update() {
    let app = init_service(create_test_app().await).await;
    for sample in get_update_samples() {
        if sample.algorithm != "sha512" {
            continue; // Skip samples that are not sha512
        }
        let req = TestRequest::post()
            .uri("/modrinth/v2/version_files/update")
            .set_json(&json!({
                "hashes": [sample.hash],
                "algorithm": sample.algorithm,
                "loaders": sample.loaders,
                "game_versions": sample.game_versions
            }))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(status.is_success(), "Status: {}, Body: {}", status, body);
    }
}


#[actix_web::test]
async fn test_modrinth_version_files_update_empty() {
    let app = init_service(create_test_app().await).await;
    let algorithm_list = ["sha1", "sha512"];
    for algorithm in algorithm_list {
        let req = TestRequest::post()
            .uri("/modrinth/v2/version_files/update")
            .set_json(&json!({
                "hashes": [],
                "algorithm": algorithm,
                "loaders": [],
                "game_versions": []
            }))
            .to_request();
        let resp = app.call(req).await.unwrap();

        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        assert!(
            status.is_success(),
            "Status: {}, Body: {}",
            status,
            body
        );
        // need 200 ok, response body is {}
        assert_eq!(body, "{}", "Expected empty object, got: {}", body);
    }
}

#[actix_web::test]
async fn test_modrinth_tag_category() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get()
        .uri("/modrinth/v2/tag/category")
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_tag_loader() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get()
        .uri("/modrinth/v2/tag/loader")
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}

#[actix_web::test]
async fn test_modrinth_tag_game_version() {
    let app = init_service(create_test_app().await).await;
    let req = TestRequest::get()
        .uri("/modrinth/v2/tag/game_version")
        .to_request();
    let resp = app.call(req).await.unwrap();

    let status = resp.status();
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes);
    assert!(status.is_success(), "Status: {}, Body: {}", status, body);
}
