use reqwest::blocking::Client;
use reqwest::StatusCode;

mod utils;
use utils::start_test_server;

#[test]
fn test_static_file_serving() {
    start_test_server(8097);
    let client = Client::new();

    let response_css = client.get("http://localhost:8097/static/styles.css").send().unwrap();
    assert_eq!(response_css.status(), StatusCode::OK);
    let headers_css = response_css.headers();
    assert_eq!(headers_css.get("Content-Type").unwrap(), "text/css");
    let content_css = response_css.text().unwrap();
    assert!(content_css.contains("body {"));

    let response_js = client.get("http://localhost:8097/static/script.js").send().unwrap();
    assert_eq!(response_js.status(), StatusCode::OK);
    let headers_js = response_js.headers();
    assert_eq!(headers_js.get("Content-Type").unwrap(), "application/javascript");
    let content_js = response_js.text().unwrap();
    assert!(content_js.contains("console.log("));
}

#[test]
fn test_static_forbidden_files() {
    start_test_server(8099);
    let client = Client::new();

    let response_php = client.get("http://localhost:8099/static/malicious.php").send().unwrap();
    assert_eq!(response_php.status(), StatusCode::FORBIDDEN);

    let response_exe = client.get("http://localhost:8099/static/malicious.exe").send().unwrap();
    assert_eq!(response_exe.status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_static_directory_listing_protection() {
    start_test_server(8100);
    let client = Client::new();

    let response = client.get("http://localhost:8100/static/").send().unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_static_large_file_rejection() {
    start_test_server(8101);

    let client = Client::new();
    let response = client.get("http://localhost:8101/static/big_file.dat").send().unwrap();

    
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[test]
fn test_static_security_headers() {
    start_test_server(8102);
    let client = Client::new();

    let response = client.get("http://localhost:8102/static/styles.css").send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert_eq!(headers.get("X-Content-Type-Options").unwrap(), "nosniff");
    assert_eq!(headers.get("X-Frame-Options").unwrap(), "DENY");
}
