use reqwest::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[tokio::test]
async fn test_post_submit() {
    start_test_server(8082).await;
    let client = Client::new();
    let response = client
        .post("http://localhost:8082/submit")
        .header("X-Mock-Subdomain", "api")
        .body("Test body content")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Received body: Test body content"));
}

#[tokio::test]
async fn test_dynamic_route() {
    start_test_server(8083).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8083/about/123")
        .header("X-Mock-Subdomain", "api")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_text = response.text().await.unwrap();
    println!("{}", response_text);
    assert!(response_text.contains("Details about ID: 123"));
}
