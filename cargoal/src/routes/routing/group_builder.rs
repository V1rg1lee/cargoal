use crate::routes::http::method::HttpMethod;
use crate::routes::http::request::Request;
use crate::routes::http::response::Response;
use crate::routes::server::server_handle::ServerHandle;
use crate::routes::routing::middleware::Middleware;
use crate::routes::routing::route_builder::RouteBuilder;
use std::sync::Arc;

/// Define the GroupBuilder struct
/// ## Fields
/// - prefix: String
/// - server: ServerHandle
/// - middlewares: Vec<Middleware>
pub struct GroupBuilder {
    prefix: String,
    server: ServerHandle,
    middlewares: Vec<Middleware>,
}

/// Implement the GroupBuilder struct
impl GroupBuilder {
    /// Create a new GroupBuilder instance
    /// ## Args
    /// - prefix: &str
    /// - server: ServerHandle
    /// ## Returns
    /// - GroupBuilder
    pub fn new(prefix: &str, server: ServerHandle) -> Self {
        Self {
            prefix: prefix.to_string(),
            server,
            middlewares: Vec::new(),
        }
    }

    /// Route a path with a method
    /// ## Args
    /// - path: &str
    /// - method: HttpMethod
    /// ## Returns
    /// - RouteBuilder
    pub fn route(&mut self, path: &str, method: HttpMethod) -> RouteBuilder {
        let full_path = format!("{}{}", self.prefix, path);
        let mut route_builder = self.server.route(&full_path, method);
    
        for middleware in &self.middlewares {
            let middleware_ref = Arc::clone(middleware);
            route_builder = route_builder.with_middleware(move |req| middleware_ref(req));
        }
    
        route_builder
    }

    /// Add a middleware to the Group
    /// ## Args
    /// - middleware: F
    /// ## Where
    /// - F: Fn(&Request) -> Option<Response> + Send + Sync + 'static
    /// ## Returns
    /// - &mut Self
    pub fn add_middleware<F>(&mut self, middleware: F) -> &mut Self
    where
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static,
    {
        self.middlewares.push(Arc::new(middleware));
        self
    }
}
