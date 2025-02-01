use reqwest::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[tokio::test]
async fn test_middleware_log() {
    start_test_server(8086).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8086/middleware-test/log")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Middleware executed!"));
}

#[tokio::test]
async fn test_global_middleware_block() {
    start_test_server(8092).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8092/middleware-block")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Forbidden by middleware"));
}

#[tokio::test]
async fn test_route_middleware_block() {
    start_test_server(8093).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8093/middleware-block-2")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Forbidden by middleware"));
}

#[tokio::test]
async fn test_group_middleware_block() {
    start_test_server(8094).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8094/middleware-block-3/block")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Forbidden by middleware"));
}
