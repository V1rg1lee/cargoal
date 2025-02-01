use crate::routes::http::method::HttpMethod;
use crate::routes::http::request::Request;
use crate::routes::http::response::Response;
use crate::routes::routing::middleware::Middleware;
use regex::Regex;

/// Define the Route struct
/// ## Fields
/// - subdomain: Option<String>
/// - path: String
/// - method: HttpMethod
/// - handler: Box<dyn Fn(Request) -> Response + Send + Sync>
/// - regex: Option<Regex>
/// - middlewares: Vec<Middleware>
pub struct Route {
    pub(crate) subdomain: Option<String>,
    pub(crate) path: String,
    pub(crate) method: HttpMethod,
    pub(crate) handler: Box<dyn Fn(Request) -> Response + Send + Sync>,
    pub(crate) regex: Option<Regex>,
    pub(crate) middlewares: Vec<Middleware>,
}
