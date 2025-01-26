use cargoal::routes::http::method::HttpMethod;
use cargoal::routes::http::response::Response;
use cargoal::routes::server::Server;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::Client;
    use reqwest::StatusCode;
    use std::thread;
    use std::time::Duration;

    fn start_test_server(port: u16) {
        thread::spawn(move || {
            test(port);
        });
        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    fn test_home_page() {
        start_test_server(8080);
        let client = Client::new();
        let response = client
            .get("http://www.localhost:8080/")
            .header("Host", "www.localhost")
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_about_page() {
        start_test_server(8081);
        let client = Client::new();
        let response = client.get("http://localhost:8081/about").send().unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_post_submit() {
        start_test_server(8082);
        let client = Client::new();
        let response = client
            .post("http://api.localhost:8082/submit")
            .header("Host", "api.localhost")
            .body("Test body content")
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.text().unwrap().contains("Received body: Test body content"));
    }

    #[test]
    fn test_dynamic_route() {
        start_test_server(8083);
        let client = Client::new();
        let response = client
            .get("http://api.localhost:8083/about/123")
            .header("Host", "api.localhost")
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.text().unwrap().contains("Details about ID: 123"));
    }

    #[test]
    fn test_group_routes() {
        start_test_server(8084);
        let client = Client::new();
        let response = client
            .get("http://api.localhost:8084/v1/users")
            .header("Host", "api.localhost")
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.text().unwrap().contains("[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]"));

        let response = client
            .get("http://api.localhost:8084/v1/users/42")
            .header("Host", "api.localhost")
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
            .request(reqwest::Method::OPTIONS, "http://api.localhost:8085/options-test")
            .header("Host", "api.localhost")
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("Allow").unwrap().to_str().unwrap(),
            "GET, POST"
        );
    }

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
            .get("http://unknown.localhost:8090/")
            .header("Host", "unknown.localhost")
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
            .get("http://api.localhost:8091/about/")
            .header("Host", "api.localhost")
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(response.text().unwrap().contains("Bad Request: ID cannot be empty"));
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
            let response = client.get("http://api.localhost:8095/v1/users").send().unwrap();
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


}


fn test(port: u16) {
    let mut app = Server::new(&format!("127.0.0.1:{}", port));

    app = app.with_template_dirs(vec!["templates"]);

    // Middleware configuration
    app.router.add_middleware(|req| {
        println!("Middleware log: Request received for path: {}", req.path);
        None
    });

    app.router.add_middleware(|req| {
        if req.path == "/middleware-block" {
            Some(Response::new(
                403,
                Some("Forbidden by middleware".to_string()),
            ))
        } else {
            None
        }
    });

    // Define routes
    app.route("/", HttpMethod::GET)
        .with_subdomain("www")
        .with_template("index")
        .with_context(|_req| {
            let mut context = HashMap::new();
            context.insert("title", "Home Page".to_string());
            context.insert("message", "Welcome to the Home Page!".to_string());
            context
        })
        .register();

    app.route("/about", HttpMethod::GET)
        .with_template("about")
        .with_context(|_req| {
            let mut context = HashMap::new();
            context.insert("title", "About Us".to_string());
            context.insert("message", "Learn more about us here.".to_string());
            context
        })
        .register();

    app.route("/query-test", HttpMethod::GET)
        .with_handler(|req| {
            if let Some(name) = req.params.get("name") {
                Response::new(
                    200,
                    Some(format!("Hello, {}!", name)),
                )
            } else {
                Response::new(
                    400,
                    Some("Missing 'name' parameter".to_string()),
                )
            }
        })
        .register();

    app.route("/submit", HttpMethod::POST)
        .with_subdomain("api")
        .with_handler(|req| {
            if let Some(body) = req.body {
                Response::new(
                    200,
                    Some(format!("Received body: {}", body)),
                )
                .with_header("Content-Type", "text/plain")
            } else {
                Response::new(
                    400,
                    Some("No body provided".to_string()),
                )
            }
        })
        .register();

    app.route("/about/:id", HttpMethod::GET)
        .with_subdomain("api")
        .with_handler(|req| {
            if let Some(id) = req.params.get("id") {
                if id.is_empty() {
                    Response::new(
                        400,
                        Some("Bad Request: ID cannot be empty".to_string()),
                    )
                } else {
                    Response::new(
                        200,
                        Some(format!("Details about ID: {}", id)),
                    )
                    .with_header("Content-Type", "application/json")
                }
            } else {
                Response::new(
                    400,
                    Some("Bad Request: Missing ID".to_string()),
                )
            }
        })
        .register();

    // Grouped routes example
    app.with_group("/v1", |group| {
        group
            .route("/users", HttpMethod::GET)
            .with_subdomain("api")
            .with_handler(|_req| {
                Response::new(
                    200,
                    Some("[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]".to_string()),
                )
                .with_header("Content-Type", "application/json")
            })
            .register();

        group
            .route("/users/:id", HttpMethod::GET)
            .with_subdomain("api")
            .with_handler(|req| {
                if let Some(id) = req.params.get("id") {
                    Response::new(
                        200,
                        Some(format!("Details about ID: {}", id)),
                    )
                    .with_header("Content-Type", "application/json")
                } else {
                    Response::new(
                        400,
                        Some("Missing user ID".to_string()),
                    )
                }
            })
            .register();
    });

    app.route("/options-test", HttpMethod::OPTIONS)
        .with_subdomain("api")
        .with_handler(|_req| {
            Response::new(
                200,
                Some("Available methods: GET, POST".to_string()),
            )
            .with_header("Allow", "GET, POST")
        })
        .register();

    app.route("/middleware-test/log", HttpMethod::GET)
        .with_handler(|req| {
            println!("Middleware log: Path accessed: {}", req.path);
            Response::new(
                200,
                Some("Middleware executed!".to_string()),
            )
        })
        .register();

    // Start the server
    println!("Server running with all routes set up.");
    app.run();
}