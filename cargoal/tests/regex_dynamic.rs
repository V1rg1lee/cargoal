use reqwest::blocking::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[test]
fn test_valid_user_id() {
    start_test_server(8080);
    let client = Client::new();
    let response = client
        .get("http://localhost:8080/v1/users/42")
        .header("X-Mock-Subdomain", "api")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().contains("Details about ID: 42"));
}

#[test]
fn test_invalid_user_id() {
    start_test_server(8081);
    let client = Client::new();
    let response = client
        .get("http://localhost:8081/v1/users/abc")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_valid_item_name() {
    start_test_server(8082);
    let client = Client::new();
    let response = client
        .get("http://localhost:8082/v1/items/widget")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .text()
        .unwrap()
        .contains("Details about item name: widget"));
}

#[test]
fn test_invalid_item_name() {
    start_test_server(8083);
    let client = Client::new();
    let response = client
        .get("http://localhost:8083/v1/items/widget123")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_valid_order_id() {
    start_test_server(8084);
    let client = Client::new();
    let response = client
        .get("http://localhost:8084/v1/orders/order-123_456")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .text()
        .unwrap()
        .contains("Details about order ID: order-123_456"));
}

#[test]
fn test_invalid_order_id() {
    start_test_server(8085);
    let client = Client::new();
    let response = client
        .get("http://localhost:8085/v1/orders/order%20id")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
