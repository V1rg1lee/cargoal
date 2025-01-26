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
        assert!(response.text().unwrap().contains("Hello, world!"));
    }

    #[test]
    fn test_about_page() {
        start_test_server(8081);
        let client = Client::new();
        let response = client.get("http://localhost:8081/about").send().unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.text().unwrap().contains("This is the about page."));
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
        assert!(response.text().unwrap().contains("Details about user ID: 42"));
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
            "text/plain"
        );
    }

    #[test]
    fn test_high_load() {
        start_test_server(8095);
        let client = Client::new();
        for _ in 0..100 {
            let response = client.get("http://localhost:8095/about").send().unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert!(response.text().unwrap().contains("This is the about page."));
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

pub fn test(port: u16) {
    let mut server = cargoal::routes::server::Server::new(&format!("127.0.0.1:{}", port));

    server.router.add_middleware(|req| {
        println!("Middleware log: Request received for path: {}", req.path);
        None
    });

    server.router.add_middleware(|req| {
        if req.path == "/middleware-block" {
            Some(cargoal::routes::http::response::Response::new(403, Some("Forbidden by middleware".to_string())))
        } else {
            None
        }
    });    

    server = server
        // Home page on www subdomain
        .with_route(
            Some("www"),
            "/",
            cargoal::routes::http::method::HttpMethod::GET,
            |req| {
                cargoal::routes::http::response::Response::new(200, Some("Hello, world! Welcome to the home page.".to_string()))
                    .with_header("Content-Type", "text/plain")
            },
            Some("Home page on www subdomain"),
        )
        // With query parameters
        .with_route(
            None,
            "/query-test",
            cargoal::routes::http::method::HttpMethod::GET,
            |req| {
                println!("Handler received params: {:?}", req.params);
                if let Some(name) = req.params.get("name") {
                    cargoal::routes::http::response::Response::new(200, Some(format!("Hello, {}!", name)))
                } else {
                    cargoal::routes::http::response::Response::new(400, Some("Missing 'name' parameter".to_string()))
                }
            },
            Some("Test query parameters"),
        )
        // Base about page
        .with_route(
            None,
            "/about",
            cargoal::routes::http::method::HttpMethod::GET,
            |req| {
                cargoal::routes::http::response::Response::new(200, Some("This is the about page.".to_string()))
                    .with_header("Content-Type", "text/plain")
            },
            Some("About page available globally"),
        )
        // POST route with body
        .with_route(
            Some("api"),
            "/submit",
            cargoal::routes::http::method::HttpMethod::POST,
            |req| {
                if let Some(body) = &req.body {
                    cargoal::routes::http::response::Response::new(200, Some(format!("Received body: {}", body)))
                        .with_header("Content-Type", "text/plain")
                } else {
                    cargoal::routes::http::response::Response::new(400, Some("No body provided".to_string()))
                }
            },
            Some("Endpoint to test request body handling"),
        )
        // Dynamic route with parameter
        .with_route(
            Some("api"),
            "/about/:id",
            cargoal::routes::http::method::HttpMethod::GET,
            |req| {
                if let Some(id) = req.params.get("id") {
                    if id.is_empty() {       
                        cargoal::routes::http::response::Response::new(400, Some("Bad Request: ID cannot be empty".to_string()))
                    } else {
                        cargoal::routes::http::response::Response::new(200, Some(format!("Details about ID: {}", req.params.get("id").unwrap())))
                            .with_header("Content-Type", "application/json")
                    }
                } else {
                    cargoal::routes::http::response::Response::new(400, Some("Bad Request: Missing ID".to_string()))
                }
            },
            Some("Fetch details about a specific ID in /about/:id"),
        )
        // Group of routes under "/v1" under "api"
        .with_group("/v1", |router| {
            router.add_route(
                Some("api"),
                "/users",
                cargoal::routes::http::method::HttpMethod::GET,
                |req| {
                    cargoal::routes::http::response::Response::new(200, Some("[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]".to_string()))
                        .with_header("Content-Type", "application/json")
                },
                Some("Get the list of users"),
            );
            router.add_route(
                Some("api"),
                "/users/:id",
                cargoal::routes::http::method::HttpMethod::GET,
                |req| {
                    if let Some(id) = req.params.get("id") {
                        cargoal::routes::http::response::Response::new(200, Some(format!("Details about user ID: {}", id)))
                            .with_header("Content-Type", "application/json")
                    } else {
                        cargoal::routes::http::response::Response::new(400, Some("Bad Request: Missing user ID".to_string()))
                    }
                },
                Some("Fetch details about a specific user ID"),
            );
        })
        // OPTIONS for "/options-test" under "api"
        .with_route(
            Some("api"),
            "/options-test",
            cargoal::routes::http::method::HttpMethod::OPTIONS,
            |req| {
                cargoal::routes::http::response::Response::new(200, Some("Available methods: GET, POST".to_string()))
                    .with_header("Allow", "GET, POST")
            },
            Some("Options endpoint for testing HTTP OPTIONS method"),
        )
        // Middleware pour journalisation (exemple simplifié)
        .with_group("/middleware-test", |router| {
            router.add_route(
                None,
                "/log",
                cargoal::routes::http::method::HttpMethod::GET,
                |req| {
                    println!("Middleware log: Path accessed: {}", req.path);
                    cargoal::routes::http::response::Response::new(200, Some("Middleware executed!".to_string()))
                },
                Some("Test middleware execution"),
            );
        });

    println!("Server running with all routes set up.");
    server.run();
}
