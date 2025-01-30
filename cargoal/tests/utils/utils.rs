use super::handlers::{
    item_handler, middleware_test_handler, options_test_handler,
    order_handler, query_test_handler, submit_handler, this_should_not_be_reached_handler,
    user_handler, users_handler,
};
use super::templates::{conditional_handler, filters_handler, include_handler, list_handler,about_handler, home_handler, escaping_handler};
use super::middlewares::{block_middleware, block_middleware_group, logging_middleware};
use cargoal::routes::http::HttpMethod;
use cargoal::routes::server::Server;
use std::thread;
use std::time::Duration;

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

    // Template dir configuration
    app.with_template_dirs(vec!["tests/templates"]);

    // Middleware configuration
    app.router.add_middleware(logging_middleware);

    app.router.add_middleware(block_middleware);

    // Define routes
    app.route("/", HttpMethod::GET)
        .with_subdomain("www")
        .with_template("home.html")
        .with_context(home_handler)
        .register();

    app.route("/conditional", HttpMethod::GET)
        .with_template("conditional.html")
        .with_context(conditional_handler)
        .register();

    app.route("/list", HttpMethod::GET)
        .with_template("list.html")
        .with_context(list_handler)
        .register();

    app.route("/filters", HttpMethod::GET)
        .with_template("filters.html")
        .with_context(filters_handler)
        .register();

    app.route("/include", HttpMethod::GET)
        .with_template("include.html")
        .with_context(include_handler)
        .register();

    app.route("/missing", HttpMethod::GET)
        .with_template("missing.html")
        .register();

    app.route("/escaping", HttpMethod::GET)
        .with_template("escaping.html")
        .with_context(escaping_handler)
        .register();

    app.route("/not_allowed", HttpMethod::GET)
        .with_template("not_allowed.txt")
        .register();

    app.route("/corrupt", HttpMethod::GET)
        .with_template("corrupt.html")
        .register();

    app.route("/injection", HttpMethod::GET)
        .with_template("injection.html")
        .register();
    
    app.route("/about", HttpMethod::GET)
        .with_template("about.html")
        .with_context(about_handler)
        .register();

    app.route("/query-test", HttpMethod::GET)
        .with_handler(query_test_handler)
        .register();

    app.route("/submit", HttpMethod::POST)
        .with_subdomain("api")
        .with_handler(submit_handler)
        .register();

    app.route("/about/:id", HttpMethod::GET)
        .with_subdomain("api")
        .with_handler(user_handler)
        .register();

    // Grouped routes example
    app.with_group("/v1", |group| {
        group
            .route("/users", HttpMethod::GET)
            .with_subdomain("api")
            .with_handler(users_handler)
            .register();

        group
            .route("/users/:id", HttpMethod::GET)
            .with_regex(r"^/v1/users/(?P<id>\d+)$")
            .with_subdomain("api")
            .with_handler(user_handler)
            .register();

        group
            .route("/orders/:order_id", HttpMethod::GET)
            .with_regex(r"^/v1/orders/(?P<order_id>[a-zA-Z0-9_-]+)$")
            .with_handler(order_handler)
            .register();

        group
            .route("/items/:name", HttpMethod::GET)
            .with_regex(r"^/v1/items/(?P<name>[a-zA-Z]+)$")
            .with_handler(item_handler)
            .register();
    });

    // Route with OPTIONS method
    app.route("/options-test", HttpMethod::OPTIONS)
        .with_subdomain("api")
        .with_handler(options_test_handler)
        .register();

    // Middleware test
    app.route("/middleware-test/log", HttpMethod::GET)
        .with_handler(middleware_test_handler)
        .register();

    app.route("/middleware-block-2", HttpMethod::GET)
        .with_middleware(block_middleware_group)
        .register();

    app.with_group("/middleware-block-3", |group| {
        group.add_middleware(block_middleware_group);

        group
            .route("/block", HttpMethod::GET)
            .with_handler(this_should_not_be_reached_handler)
            .register();
    });

    // Start the server
    println!("Server running with all routes set up.");
    app.run();
}
