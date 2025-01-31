use reqwest::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[tokio::test]
async fn test_home_page() {
    start_test_server(8080).await;
    let client = Client::new();
    let response = client
        .get("http://localhost:8080/")
        .header("X-Mock-Subdomain", "www")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().await.unwrap();

    assert!(content.contains("<h1>Home Page</h1>"));
    assert!(content.contains("<p>Welcome to the Home Page!</p>"));

    assert!(!content.contains("Error rendering template"));
}

#[tokio::test]
async fn test_about_page() {
    start_test_server(8081).await;
    let client = Client::new();
    let response = client.get("http://localhost:8081/about").send().await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().await.unwrap();

    assert!(content.contains("<h1>About Us</h1>"));
    assert!(content.contains("<p>Learn more about us here.</p>"));
}

#[tokio::test]
async fn test_template_conditionals() {
    start_test_server(8082).await;
    let client = Client::new();
    let response = client.get("http://localhost:8082/conditional").send().await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().await.unwrap();

    if content.contains("You are logged in.") {
        assert!(!content.contains("Please log in."));
    } else {
        assert!(content.contains("Please log in."));
    }
}

#[tokio::test]
async fn test_template_loops() {
    start_test_server(8083).await;
    let client = Client::new();
    let response = client.get("http://localhost:8083/list").send().await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().await.unwrap();

    assert!(content.contains("<li>Item 1</li>"));
    assert!(content.contains("<li>Item 2</li>"));
    assert!(content.contains("<li>Item 3</li>"));
}

#[tokio::test]
async fn test_template_filters() {
    start_test_server(8084).await;
    let client = Client::new();
    let response = client.get("http://localhost:8084/filters").send().await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().await.unwrap();

    assert!(content.contains("<p>UPPERCASE TEXT</p>"));
    assert!(content.contains("<p>lowercase text</p>"));
    assert!(content.contains("<p>Trimmed Text</p>"));
}

#[tokio::test]
async fn test_template_includes() {
    start_test_server(8085).await;
    let client = Client::new();
    let response = client.get("http://localhost:8085/include").send().await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().await.unwrap();

    assert!(content.contains("Header Section"));
    assert!(content.contains("Main Content Section"));
    assert!(content.contains("Footer Section"));
}

#[tokio::test]
async fn test_missing_template() {
    start_test_server(8090).await;
    let client = Client::new();
    let response = client.get("http://localhost:8090/missing").send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let content = response.text().await.unwrap();
    assert!(content.contains("Template 'missing.html' not found!"));
}

#[tokio::test]
async fn test_html_escaping() {
    start_test_server(8091).await;
    let client = Client::new();
    let response = client.get("http://localhost:8091/escaping").send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().await.unwrap();
    assert!(content.contains("&lt;script&gt;alert(&#x27;XSS&#x27;)&lt;&#x2f;script&gt;"));
}

#[tokio::test]
async fn test_only_html_files_loaded() {
    start_test_server(8093).await;
    let client = Client::new();

    let response_html = client.get("http://localhost:8093/escaping").send().await.unwrap();
    assert_eq!(response_html.status(), StatusCode::OK);

    let response_txt = client.get("http://localhost:8093/not_allowed").send().await.unwrap();
    assert_eq!(response_txt.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_injection_protection() {
    start_test_server(8095).await;
    let client = Client::new();
    let response = client.get("http://localhost:8095/injection").send().await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}