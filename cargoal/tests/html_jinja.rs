use reqwest::blocking::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[test]
fn test_home_page() {
    start_test_server(8080);
    let client = Client::new();
    let response = client
        .get("http://localhost:8080/")
        .header("X-Mock-Subdomain", "www")
        .send()
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().unwrap();

    assert!(content.contains("<h1>Home Page</h1>"));
    assert!(content.contains("<p>Welcome to the Home Page!</p>"));

    assert!(!content.contains("Error rendering template"));
}

#[test]
fn test_about_page() {
    start_test_server(8081);
    let client = Client::new();
    let response = client.get("http://localhost:8081/about").send().unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().unwrap();

    assert!(content.contains("<h1>About Us</h1>"));
    assert!(content.contains("<p>Learn more about us here.</p>"));
}

#[test]
fn test_template_conditionals() {
    start_test_server(8082);
    let client = Client::new();
    let response = client.get("http://localhost:8082/conditional").send().unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().unwrap();

    if content.contains("You are logged in.") {
        assert!(!content.contains("Please log in."));
    } else {
        assert!(content.contains("Please log in."));
    }
}

#[test]
fn test_template_loops() {
    start_test_server(8083);
    let client = Client::new();
    let response = client.get("http://localhost:8083/list").send().unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().unwrap();

    assert!(content.contains("<li>Item 1</li>"));
    assert!(content.contains("<li>Item 2</li>"));
    assert!(content.contains("<li>Item 3</li>"));
}

#[test]
fn test_template_filters() {
    start_test_server(8084);
    let client = Client::new();
    let response = client.get("http://localhost:8084/filters").send().unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().unwrap();

    assert!(content.contains("<p>UPPERCASE TEXT</p>"));
    assert!(content.contains("<p>lowercase text</p>"));
    assert!(content.contains("<p>Trimmed Text</p>"));
}

#[test]
fn test_template_includes() {
    start_test_server(8085);
    let client = Client::new();
    let response = client.get("http://localhost:8085/include").send().unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().unwrap();

    assert!(content.contains("Header Section"));
    assert!(content.contains("Main Content Section"));
    assert!(content.contains("Footer Section"));
}

#[test]
fn test_missing_template() {
    start_test_server(8090);
    let client = Client::new();
    let response = client.get("http://localhost:8090/missing").send().unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let content = response.text().unwrap();
    assert!(content.contains("Template 'missing.html' not found!"));
}

#[test]
fn test_html_escaping() {
    start_test_server(8091);
    let client = Client::new();
    let response = client.get("http://localhost:8091/escaping").send().unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content = response.text().unwrap();
    assert!(content.contains("&lt;script&gt;alert(&#x27;XSS&#x27;)&lt;&#x2f;script&gt;"));
}

#[test]
fn test_only_html_files_loaded() {
    start_test_server(8093);
    let client = Client::new();

    let response_html = client.get("http://localhost:8093/escaping").send().unwrap();
    assert_eq!(response_html.status(), StatusCode::OK);

    let response_txt = client.get("http://localhost:8093/not_allowed").send().unwrap();
    assert_eq!(response_txt.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_injection_protection() {
    start_test_server(8095);
    let client = Client::new();
    let response = client.get("http://localhost:8095/injection").send().unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}