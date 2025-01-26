use super::super::http::method::HttpMethod;
use super::super::server::Server;
use super::routeBuilder::RouteBuilder;

/// Define the GroupBuilder struct
/// ## Fields
/// - prefix: String
/// - server: &'a mut Server
pub struct GroupBuilder<'a> {
    prefix: String,
    server: &'a mut Server,
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
        }
    }

    /// Route a path with a method
    /// ## Args
    /// - path: &str
    /// - method: HttpMethod
    /// ## Returns
    /// - RouteBuilder
    pub fn route(&mut self, path: &str, method: HttpMethod) -> RouteBuilder<'_> {
        let full_path = format!("{}{}", self.prefix, path); // Alloue directement
        self.server.route(Box::leak(full_path.into_boxed_str()), method)
    }
}