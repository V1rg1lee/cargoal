use super::super::http::request::Request;
use super::super::http::response::Response;
use std::sync::Arc;

/// Define the Middleware type
pub type Middleware = Arc<dyn Fn(&Request) -> Option<Response> + Send + Sync>;