use reqwest::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[tokio::test]
async fn test_group_routes() {
    start_test_server(8084).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8084/v1/users")
        .header("X-Mock-Subdomain", "api")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]"));

    let response = client
        .get("http://localhost:8084/v1/users/42")
        .header("X-Mock-Subdomain", "api")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains("Details about ID: 42"));
}

#[tokio::test]
async fn test_options_method() {
    start_test_server(8085).await;
    let client = Client::new();
    let response = client
        .request(
            reqwest::Method::OPTIONS,
            "http://localhost:8085/options-test",
        )
        .header("X-Mock-Subdomain", "api")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Allow").unwrap().to_str().unwrap(),
        "GET, POST"
    );
}

#[tokio::test]
async fn test_unknown_route() {
    start_test_server(8087).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8087/unknown-route")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.text().await.unwrap().contains("Not Found"));
}

#[tokio::test]
async fn test_redirect_slash_to_non_slash() {
    start_test_server(8089).await;
    let client = Client::new();
    let response = client.get("http://localhost:8089/about/").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.url().as_str(), "http://localhost:8089/about");
}

#[tokio::test]
async fn test_unknown_subdomain() {
    start_test_server(8090).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8090/")
        .header("X-Mock-Subdomain", "unknown")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.text().await.unwrap().contains("Not Found"));
}

#[tokio::test]
async fn test_bad_request_missing_param() {
    start_test_server(8091).await;
    let client = Client::new();

    // Test with missing parameter
    let response = client
        .get("http://localhost:8091/about/")
        .header("X-Mock-Subdomain", "api")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.text().await.unwrap().contains("Not Found"));
}

#[tokio::test]
async fn test_method_not_allowed() {
    start_test_server(8093).await;
    let client = Client::new();
    let response = client.delete("http://localhost:8093/about").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(
        response.headers().get("Allow").unwrap().to_str().unwrap(),
        "GET"
    );
}

#[tokio::test]
async fn test_custom_headers() {
    start_test_server(8094).await;
    let client = Client::new();
    let response = client.get("http://localhost:8094/about").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap(),
        "text/html"
    );
}

#[tokio::test]
async fn test_high_load() {
    start_test_server(8095).await;
    let client = Client::new();
    for _ in 0..100 {
        let response = client
            .get("http://localhost:8095/v1/users")
            .header("X-Mock-Subdomain", "api")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_query_parameters() {
    start_test_server(8096).await;
    let client = reqwest::Client::new();

    // Test with a single parameter
    let response = client
        .get("http://localhost:8096/query-test?name=Alice")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains("Hello, Alice!"));

    // Test with missing parameter
    let response = client
        .get("http://localhost:8096/query-test")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(response
        .text()
        .await
        .unwrap()
        .contains("Missing 'name' parameter"));

    // Test with multiple parameters
    let response = client
        .get("http://localhost:8096/query-test?name=Bob&age=25")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains("Hello, Bob!"));

    // Test with empty parameter
    let response = client
        .get("http://localhost:8096/query-test?name=")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains("Hello, !"));
}
