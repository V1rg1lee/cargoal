use cargoal::routes::http::response::Response;
use cargoal::routes::http::request::Request;

#[cfg(test)]
pub fn logging_middleware(req: &Request) -> Option<Response> {
    println!("Middleware log: Request received for path: {}", req.path);
    None
}

#[cfg(test)]
pub fn block_middleware(req: &Request) -> Option<Response> {
    if req.path == "/middleware-block" {
        Some(Response::new(403, Some("Forbidden by middleware".to_string())))
    } else {
        None
    }
}

#[cfg(test)]
pub fn block_middleware_group(_req: &Request) -> Option<Response> {
    Some(Response::new(403, Some("Forbidden by middleware".to_string())))
}