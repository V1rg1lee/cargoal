use reqwest::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[tokio::test]
async fn test_valid_user_id() {
    start_test_server(8080).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8080/v1/users/42")
        .header("X-Mock-Subdomain", "api")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains("Details about ID: 42"));
}

#[tokio::test]
async fn test_invalid_user_id() {
    start_test_server(8081).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8081/v1/users/abc")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_valid_item_name() {
    start_test_server(8082).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8082/v1/items/widget")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Details about item name: widget"));
}

#[tokio::test]
async fn test_invalid_item_name() {
    start_test_server(8083).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8083/v1/items/widget123")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_valid_order_id() {
    start_test_server(8084).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8084/v1/orders/order-123_456")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Details about order ID: order-123_456"));
}

#[tokio::test]
async fn test_invalid_order_id() {
    start_test_server(8085).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8085/v1/orders/order%20id")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
