use actix_web::{
    dev::Service,
    test::{init_service, TestRequest},
};

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
