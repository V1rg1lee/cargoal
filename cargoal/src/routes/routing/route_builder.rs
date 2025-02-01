use crate::routes::server::server_handle::ServerHandle;
use crate::routes::http::method::HttpMethod;
use crate::routes::http::request::Request;
use crate::routes::http::response::Response;
use crate::routes::routing::middleware::Middleware;
use std::collections::HashMap;
use std::sync::Arc;
use minijinja::Value;

type ContextFn = Box<dyn Fn(&Request) -> HashMap<String, Value> + Send + Sync>;

/// Define the RouteBuilder struct
/// ## Fields
/// - path: String
/// - method: HttpMethod
/// - template: Option<String>
/// - context_fn: Option<ContextFn>
/// - server: ServerHandle
/// - handler: Option<Box<dyn Fn(Request) -> Response + Send + Sync>>
/// - subdomain: Option<String>
/// - regex: Option<String>
/// - middlewares: Vec<Middleware>
pub struct RouteBuilder {
    path: String,
    method: HttpMethod,
    template: Option<String>,
    context_fn: Option<ContextFn>,
    server: ServerHandle,
    handler: Option<Box<dyn Fn(Request) -> Response + Send + Sync>>,
    subdomain: Option<String>,
    regex: Option<String>,
    middlewares: Vec<Middleware>,
}

/// Implement the RouteBuilder struct
impl RouteBuilder {
    /// Create a new RouteBuilder instance
    /// ## Args
    /// - path: &str
    /// - method: HttpMethod
    /// - server: ServerHandle
    /// ## Returns
    /// - RouteBuilder
    pub fn new(path:  &str, method: HttpMethod, server: ServerHandle) -> Self {
        Self {
            path: path.to_string(),
            method,
            template: None,
            context_fn: None,
            server,
            handler: None,
            subdomain: None,
            regex: None,
            middlewares: Vec::new(),
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
    /// - template_name: &str
    /// ## Returns
    /// - RouteBuilder
    pub fn with_template(mut self, template_name: &str) -> Self {
        self.template = Some(template_name.to_string());
        self
    }

    /// Set the context function for the Route
    /// ## Args
    /// - self
    /// - context_fn: F
    /// ## Where
    /// - F: Fn(&Request) -> HashMap<String, V> + Send + Sync + 'static
    /// - V: Into<Value>
    /// ## Returns
    /// - RouteBuilder
    pub fn with_context<F, V>(mut self, context_fn: F) -> Self
    where
        F: Fn(&Request) -> HashMap<String, V> + Send + Sync + 'static,
        V: Into<Value>,
    {
        self.context_fn = Some(Box::new(move |req| {
            context_fn(req)
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect()
        }));
        self
    }

    /// Set the regex for the Route
    /// ## Args
    /// - self
    /// - regex: &str
    /// ## Returns
    /// - RouteBuilder
    pub fn with_regex(mut self, regex: &str) -> Self {
        self.regex = Some(regex.to_string());
        self
    }

    /// Add a Middleware to the Route
    /// ## Args
    /// - self
    /// - middleware: F
    /// ## Where
    /// - F: Fn(&Request) -> Option<Response> + Send + Sync + 'static
    /// ## Returns
    /// - RouteBuilder
    pub fn with_middleware<F>(mut self, middleware: F) -> Self
    where
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static,
    {
        self.middlewares.push(Arc::new(middleware));
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
    pub async fn register(self) {
        let template = self.template.map(|t| t.to_string()).clone();
        let context_fn = self.context_fn;
        let handler = self.handler;
        let subdomain = self.subdomain;
        let regex = self.regex;
        let middlewares = self.middlewares.clone();

        // Prepare the route
        let renderer = self.server.get_template_renderer().await;
        let path = self.path.to_string();
        let method = self.method.clone();

        // Prepare the router
        let router_handle = self.server.router();
        let mut router = router_handle.write().await;

        // Add the route to the server
        router.add_route(
            subdomain.as_deref(),
            &path,
            method,
            move |req: Request| {
                for middleware in &middlewares {
                    if let Some(response) = middleware(&req) {
                        return response;
                    }
                }

                // If a handler is set, use it
                if let Some(handler) = &handler {
                    return handler(req);
                }

                // If a template is set, render it
                let context = context_fn.as_ref().map_or_else(HashMap::new, |f| f(&req));
                let rendered = match template.clone() {
                    Some(t) => match renderer.render(&t, &context) {
                        Ok(output) => return Response::new(200, Some(output)).with_header("Content-Type", "text/html"),
                        Err(err) => {
                            eprintln!("Error rendering template '{}': {}", t, err);
                            if err.contains("not found") {
                                return Response::new(404, Some(format!("Template '{}' not found!", t)))
                                    .with_header("Content-Type", "text/html");
                            }
                            return Response::new(500, Some(format!("Internal Server Error: {}", err)))
                                .with_header("Content-Type", "text/html");
                        }
                    },
                    None => "Template not set.".to_string(),
                };
                

                Response::new(500, Some(rendered)).with_header("Content-Type", "text/html")
            },
            regex.as_deref(),
        );
    }
}
