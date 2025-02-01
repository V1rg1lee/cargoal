use crate::routes::http::request::Request;
use crate::routes::http::response::Response;
use std::sync::Arc;

/// Define the Middleware type
pub(crate) type Middleware = Arc<dyn Fn(&Request) -> Option<Response> + Send + Sync>;
