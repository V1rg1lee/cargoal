use super::super::http::method::HttpMethod;
use super::super::http::request::Request;
use super::super::http::response::Response;
use regex::Regex;

/// Define the Route struct
/// ## Fields
/// - subdomain: Option<String>
/// - path: String
/// - method: HttpMethod
/// - handler: Box<dyn Fn(Request) -> Response + Send + Sync>
/// - regex: Option<Regex>
pub struct Route {
    pub subdomain: Option<String>,
    pub path: String,
    pub method: HttpMethod,
    pub handler: Box<dyn Fn(Request) -> Response + Send + Sync>,
    pub regex: Option<Regex>,
}