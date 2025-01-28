use reqwest::blocking::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[test]
fn test_group_routes() {
    start_test_server(8084);
    let client = Client::new();
    let response = client
        .get("http://localhost:8084/v1/users")
        .header("X-Mock-Subdomain", "api")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().contains("[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]"));

    let response = client
        .get("http://localhost:8084/v1/users/42")
        .header("X-Mock-Subdomain", "api")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().contains("Details about ID: 42"));
}

#[test]
fn test_options_method() {
    start_test_server(8085);
    let client = Client::new();
    let response = client
        .request(reqwest::Method::OPTIONS, "http://localhost:8085/options-test")
        .header("X-Mock-Subdomain", "api")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Allow").unwrap().to_str().unwrap(),
        "GET, POST"
    );
}

#[test]
fn test_unknown_route() {
    start_test_server(8087);
    let client = Client::new();
    let response = client
        .get("http://localhost:8087/unknown-route")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.text().unwrap().contains("Not Found"));
}

#[test]
fn test_redirect_slash_to_non_slash() {
    start_test_server(8089);
    let client = Client::new();
    let response = client
        .get("http://localhost:8089/about/")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.url().as_str(),
        "http://localhost:8089/about"
    );
}

#[test]
fn test_unknown_subdomain() {
    start_test_server(8090);
    let client = Client::new();
    let response = client
        .get("http://localhost:8090/")
        .header("X-Mock-Subdomain", "unknown")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.text().unwrap().contains("Not Found"));
}


#[test]
fn test_bad_request_missing_param() {
    start_test_server(8091);
    let client = Client::new();

    // Test with missing parameter
    let response = client
        .get("http://localhost:8091/about/")
        .header("X-Mock-Subdomain", "api")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.text().unwrap().contains("Not Found"));
}

#[test]
fn test_method_not_allowed() {
    start_test_server(8093);
    let client = Client::new();
    let response = client
        .delete("http://localhost:8093/about")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(
        response.headers().get("Allow").unwrap().to_str().unwrap(),
        "GET"
    );
}

#[test]
fn test_custom_headers() {
    start_test_server(8094);
    let client = Client::new();
    let response = client
        .get("http://localhost:8094/about")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap().to_str().unwrap(),
        "text/html"
    );
}

#[test]
fn test_high_load() {
    start_test_server(8095);
    let client = Client::new();
    for _ in 0..100 {
        let response = client
            .get("http://localhost:8095/v1/users")
            .header("X-Mock-Subdomain", "api")
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[test]
fn test_query_parameters() {
    start_test_server(8096);
    let client = reqwest::blocking::Client::new();

    // Test with a single parameter
    let response = client
        .get("http://localhost:8096/query-test?name=Alice")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().contains("Hello, Alice!"));

    // Test with missing parameter
    let response = client
        .get("http://localhost:8096/query-test")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(response.text().unwrap().contains("Missing 'name' parameter"));

    // Test with multiple parameters
    let response = client
        .get("http://localhost:8096/query-test?name=Bob&age=25")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().contains("Hello, Bob!"));

    // Test with empty parameter
    let response = client
        .get("http://localhost:8096/query-test?name=")
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().contains("Hello, !"));
}