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
