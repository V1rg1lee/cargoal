use cargoal::routes::http::response::Response;
use cargoal::routes::http::request::Request;
use std::collections::HashMap;

#[cfg(test)]
pub fn home_handler(_req: &Request) -> HashMap<&'static str, String> {
    let mut context = HashMap::new();
    context.insert("title", "Home Page".to_string());
    context.insert("message", "Welcome to the Home Page!".to_string());
    context
}

#[cfg(test)]
pub fn about_handler(_req: &Request) -> HashMap<&'static str, String> {
    let mut context = HashMap::new();
    context.insert("title", "About Us".to_string());
    context.insert("message", "Learn more about us here.".to_string());
    context
}

#[cfg(test)]
pub fn query_test_handler(req: Request) -> Response {
    if let Some(name) = req.params.get("name") {
        Response::new(200, Some(format!("Hello, {}!", name)))
    } else {
        Response::new(400, Some("Missing 'name' parameter".to_string()))
    }
}

#[cfg(test)]
pub fn submit_handler(req: Request) -> Response {
    if let Some(body) = req.body.clone() {
        Response::new(200, Some(format!("Received body: {}", body)))
            .with_header("Content-Type", "text/plain")
    } else {
        Response::new(400, Some("No body provided".to_string()))
    }
}

#[cfg(test)]
pub fn users_handler(_req: Request) -> Response {
    Response::new(
        200,
        Some("[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]".to_string()),
    )
    .with_header("Content-Type", "application/json")
}


#[cfg(test)]
pub fn user_handler(req: Request) -> Response {
    if let Some(id) = req.params.get("id") {
        Response::new(200, Some(format!("Details about ID: {}", id)))
            .with_header("Content-Type", "application/json")
    } else {
        Response::new(400, Some("Bad Request: Missing ID".to_string()))
    }
}

#[cfg(test)]
pub fn options_test_handler(_req: Request) -> Response {
    Response::new(
        200,
        Some("Available methods: GET, POST".to_string()),
    )
    .with_header("Allow", "GET, POST")
}

#[cfg(test)]
pub fn order_handler(req: Request) -> Response {
    if let Some(order_id) = req.params.get("order_id") {
        Response::new(200, Some(format!("Details about order ID: {}", order_id)))
    } else {
        Response::new(400, Some("Missing order ID".to_string()))
    }
}

#[cfg(test)]
pub fn item_handler(req: Request) -> Response {
    if let Some(name) = req.params.get("name") {
        Response::new(200, Some(format!("Details about item name: {}", name)))
    } else {
        Response::new(400, Some("Missing item name".to_string()))
    }
}

#[cfg(test)]
pub fn middleware_test_handler(_req: Request) -> Response {
    Response::new(200, Some("Middleware executed!".to_string()))
}

#[cfg(test)]
pub fn this_should_not_be_reached_handler(_req: Request) -> Response {
    Response::new(200, Some("This should not be reached".to_string()))
}