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
fn test_middleware_block() {
    start_test_server(8092);
    let client = Client::new();
    let response = client
        .get("http://localhost:8092/middleware-block")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.text().unwrap().contains("Forbidden by middleware"));
}