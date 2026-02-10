use actix_web::{
    dev::Service,
    test::{TestRequest, init_service},
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

#[actix_web::test]
async fn test_curseforge_avatar_cdn() {
    let app = init_service(create_test_app().await).await;
    let sample_avatars = &[
        "/avatars/thumbnails/29/69/256/256/635838945588716414.jpeg",
        "/avatars/6/38/635351497437388438.png",
    ];

    for url in sample_avatars.iter() {
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
async fn test_modrinth_avatar_cdn() {
    let app = init_service(create_test_app().await).await;
    let sample_avatars = &["/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp"];
    for url in sample_avatars.iter() {
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
