use super::super::http::method::HttpMethod;
use super::super::server::Server;
use super::route_builder::RouteBuilder;
use super::middleware::Middleware;
use std::sync::Arc;

/// Define the GroupBuilder struct
/// ## Fields
/// - prefix: String
/// - server: &'a mut Server
pub struct GroupBuilder<'a> {
    prefix: String,
    server: &'a mut Server,
    middlewares: Vec<Middleware>,
}

/// Implement the GroupBuilder struct
/// ## Methods
impl<'a> GroupBuilder<'a> {
    /// Create a new GroupBuilder instance
    /// ## Args
    /// - prefix: &str
    /// - server: &'a mut Server
    /// ## Returns
    /// - GroupBuilder
    pub fn new(prefix: &str, server: &'a mut Server) -> Self {
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
    pub fn route(&mut self, path: &str, method: HttpMethod) -> RouteBuilder<'_> {
        let full_path = format!("{}{}", self.prefix, path); 
        let mut route_builder = self.server.route(Box::leak(full_path.into_boxed_str()), method);
    
        for middleware in &self.middlewares {
            route_builder = route_builder.with_middleware(Arc::clone(middleware));
        }
    
        route_builder
    }

    /// Add a middleware to the Group
    /// ## Args
    /// - middleware: Middleware
    /// ## Returns
    /// - &mut Self
    pub fn add_middleware(&mut self, middleware: Middleware) -> &mut Self
    {
        self.middlewares.push(middleware);
        self
    }
}