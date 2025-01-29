use super::super::http::Request;
use super::super::http::Response;
use std::sync::Arc;

/// Define the Middleware type
pub(crate) type Middleware = Arc<dyn Fn(&Request) -> Option<Response> + Send + Sync>;