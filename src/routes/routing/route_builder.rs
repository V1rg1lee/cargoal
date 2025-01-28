use super::super::http::method::HttpMethod;
use super::super::http::request::Request;
use super::super::http::response::Response;
use super::super::server::Server;
use std::collections::HashMap;

/// Define the RouteBuilder struct
/// ## Fields
/// - path: &'a str
/// - method: HttpMethod
/// - template: Option<&'a str>
/// - context_fn: Option<Box<dyn Fn(&Request) -> HashMap<&'static str, String> + Send + Sync>>
/// - server: &'a mut Server
/// - handler: Option<Box<dyn Fn(Request) -> Response + Send + Sync>>
/// - subdomain: Option<String>
/// - regex: Option<&'a str>
pub struct RouteBuilder<'a> {
    path: &'a str,
    method: HttpMethod,
    template: Option<&'a str>,
    context_fn: Option<Box<dyn Fn(&Request) -> HashMap<&'static str, String> + Send + Sync>>,
    server: &'a mut Server,
    handler: Option<Box<dyn Fn(Request) -> Response + Send + Sync>>,
    subdomain: Option<String>,
    regex: Option<&'a str>,
}

/// Implement the RouteBuilder struct
impl<'a> RouteBuilder<'a> {
    /// Create a new RouteBuilder instance
    /// ## Args
    /// - path: &'a str
    /// - method: HttpMethod
    /// - server: &'a mut Server
    /// ## Returns
    /// - RouteBuilder
    pub fn new(path: &'a str, method: HttpMethod, server: &'a mut Server) -> Self {
        Self {
            path,
            method,
            template: None,
            context_fn: None,
            server,
            handler: None,
            subdomain: None,
            regex: None,
        }
    }

    /// Set the subdomain for the Route
    /// ## Args
    /// - self
    /// - subdomain: &str
    /// ## Returns
    /// - RouteBuilder
    pub fn with_subdomain(mut self, subdomain: &str) -> Self {
        self.subdomain = Some(subdomain.to_string());
        self
    }

    /// Set the template for the Route
    /// ## Args
    /// - self
    /// - template_name: &'a str
    /// ## Returns
    /// - RouteBuilder
    pub fn with_template(mut self, template_name: &'a str) -> Self {
        self.template = Some(template_name);
        self
    }

    /// Set the context function for the Route
    /// ## Args
    /// - self
    /// - context_fn: F
    /// ## Where
    /// - F: Fn(&Request) -> HashMap<&'static str, String> + Send + Sync + 'static
    /// ## Returns
    /// - RouteBuilder
    pub fn with_context<F>(mut self, context_fn: F) -> Self
    where
        F: Fn(&Request) -> HashMap<&'static str, String> + Send + Sync + 'static,
    {
        self.context_fn = Some(Box::new(context_fn));
        self
    }

    /// Set the regex for the Route
    /// ## Args
    /// - self
    /// - regex: &'a str
    /// ## Returns
    /// - RouteBuilder
    pub fn with_regex(mut self, regex: &'a str) -> Self {
        self.regex = Some(regex);
        self
    }

    /// Set the handler for the Route
    /// ## Args
    /// - self
    /// - handler: F
    /// ## Where
    /// - F: Fn(Request) -> Response + Send + Sync + 'static
    /// ## Returns
    /// - RouteBuilder
    pub fn with_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }

    /// Register the Route with the Server
    /// ## Args
    /// - self
    /// ## Side Effects
    /// - Adds the Route to the Server's Router
    pub fn register(self) {
        let template = self.template.map(|t| t.to_string());
        let context_fn = self.context_fn;
        let handler = self.handler;
        let subdomain = self.subdomain;
        let regex = self.regex;
    
        // Prepare the route
        let renderer = self.server.get_template_renderer(); // Obtenez un renderer
        let path = self.path.to_string();
        let method = self.method.clone();
    
        // Add the route to the server
        self.server.router.add_route(
            subdomain.as_deref(),
            &path,
            method,
            move |req: Request| {
                // If a handler is set, use it
                if let Some(handler) = &handler {
                    return handler(req);
                }
    
                // If a template is set, render it
                let context = context_fn
                    .as_ref()
                    .map_or_else(HashMap::new, |f| f(&req));
                let context: HashMap<&str, &str> = context.iter().map(|(k, v)| (*k, v.as_str())).collect();
                let rendered = template
                    .as_ref()
                    .map_or_else(
                        || "Template not set.".to_string(),
                        |t| renderer.render(t, &context).unwrap_or("Error rendering template.".to_string()),
                    );
    
                Response::new(200, Some(rendered)).with_header("Content-Type", "text/html")
            },
            regex.as_deref(),
        );
    } 
}
