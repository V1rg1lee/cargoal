use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use cargoal::routes::server::Server;
use cargoal::routes::http::method::HttpMethod;
use cargoal::routes::http::response::Response;

#[cfg(test)]
pub fn start_test_server(port: u16) {
    thread::spawn(move || {
        test(port);
    });
    thread::sleep(Duration::from_secs(1));
}

#[cfg(test)]
fn test(port: u16) {
    let mut app = Server::new(&format!("127.0.0.1:{}", port));

    app = app.with_template_dirs(vec!["tests/templates"]);

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
                Response::new(
                    200,
                    Some(format!("Details about ID: {}", id)),
                )
                .with_header("Content-Type", "application/json")
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
            .with_regex(r"^/v1/users/(?P<id>\d+)$")
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

        group
            .route("/orders/:order_id", HttpMethod::GET)
            .with_regex(r"^/v1/orders/(?P<order_id>[a-zA-Z0-9_-]+)$")
            .with_handler(|req| {
                if let Some(order_id) = req.params.get("order_id") {
                    Response::new(
                        200,
                        Some(format!("Details about order ID: {}", order_id)),
                    )
                } else {
                    Response::new(400, Some("Missing order ID".to_string()))
                }
            })
            .register();

        group
            .route("/items/:name", HttpMethod::GET)
            .with_regex(r"^/v1/items/(?P<name>[a-zA-Z]+)$")
            .with_handler(|req| {
                if let Some(name) = req.params.get("name") {
                    Response::new(
                        200,
                        Some(format!("Details about item name: {}", name)),
                    )
                } else {
                    Response::new(400, Some("Missing item name".to_string()))
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