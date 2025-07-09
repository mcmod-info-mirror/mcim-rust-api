use actix_web::{
    dev::Service,
    test::{init_service, TestRequest},
};
use mcim_rust_api::test_utils::create_test_app;

const CACHED_MODRINTH_SAMPLE: &[&str] = &[
    "/data/Ua7DFN59/versions/xET3UZBe/YungsApi-1.19.2-Forge-3.8.2.jar",
    "/data/Ua7DFN59/versions/k1OTLc33/YungsApi-1.20-Fabric-4.0.4.jar",
];

const CACHED_CURSEFORGE_SAMPLE: &[&str] = &[
    "/files/6000/080/sodium-fabric-0.6.5%2Bmc1.21.1.jar",
    "/files/5217/345/Vanilla-Expanded-1.20.1-forge.jar",
    "/files/5503/516/comforts-forge-6.4.0%2B1.20.1.jar",
];

#[actix_web::test]
async fn test_modrinth_file_cdn() {
    let app = init_service(create_test_app().await).await;

    for url in CACHED_MODRINTH_SAMPLE.iter() {
        let req = TestRequest::get().uri(url).to_request();
        let resp = app.call(req).await.unwrap();
        let status = resp.status().as_u16();
        assert!((300..=400).contains(&status), "status code: {}", status);
        assert!(
            resp.headers().get("Location").is_some(),
            "Location header missing for {}",
            url
        );
    }
}

#[actix_web::test]
async fn test_curseforge_file_cdn() {
    let app = init_service(create_test_app().await).await;
    for url in CACHED_CURSEFORGE_SAMPLE.iter() {
        let req = TestRequest::get().uri(url).to_request();
        let resp = app.call(req).await.unwrap();
        let status = resp.status().as_u16();
        assert!((300..=400).contains(&status), "status code: {}", status);
        assert!(
            resp.headers().get("Location").is_some(),
            "Location header missing for {}",
            url
        );
    }
}
