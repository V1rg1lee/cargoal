use reqwest::blocking::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[test]
fn test_middleware_log() {
    start_test_server(8086);
    let client = Client::new();
    let response = client
        .get("http://localhost:8086/middleware-test/log")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().contains("Middleware executed!"));
}

#[test]
fn test_global_middleware_block() {
    start_test_server(8092);
    let client = Client::new();
    let response = client
        .get("http://localhost:8092/middleware-block")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.text().unwrap().contains("Forbidden by middleware"));
}

#[test]
fn test_route_middleware_block() {
    start_test_server(8093);
    let client = Client::new();
    let response = client
        .get("http://localhost:8093/middleware-block-2")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.text().unwrap().contains("Forbidden by middleware"));
}

#[test]
fn test_group_middleware_block() {
    start_test_server(8094);
    let client = Client::new();
    let response = client
        .get("http://localhost:8094/middleware-block-3/block")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.text().unwrap().contains("Forbidden by middleware"));
}
