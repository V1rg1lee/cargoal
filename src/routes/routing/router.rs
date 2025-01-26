use super::route::Route;
use super::super::http::method::HttpMethod;
use super::super::http::request::Request;
use super::super::http::response::Response;

/// Define the Middleware type
pub type Middleware = Box<dyn Fn(&Request) -> Option<Response> + Send + Sync>;

/// Define the Router struct
/// ## Fields
/// - routes: Vec<Route>
/// - middlewares: Vec<Middleware>
pub struct Router {
    pub routes: Vec<Route>,
    pub middlewares: Vec<Middleware>,
}

/// Implement the Router struct
impl Router {
    /// Create a new Router instance
    /// ## Returns
    /// - Router
    pub fn new() -> Self {
        Self { routes: Vec::new(), middlewares: Vec::new() }
    }

    /// Add a route to the Router
    /// ## Args
    /// - subdomain: Option<&str>
    /// - path: &str
    /// - method: HttpMethod
    /// - handler: F
    /// - description: Option<&str>
    /// ## Where
    /// - F: Fn(Request) -> Response + Send + Sync + 'static
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Adds a Route to the Router
    pub fn add_route<F>(&mut self, subdomain: Option<&str>, path: &str, method: HttpMethod, handler: F, description: Option<&str>)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        self.routes.push(Route {
            subdomain: subdomain.map(|s| s.to_string()),
            path: path.to_string(),
            method,
            handler: Box::new(handler),
            description: description.map(String::from),
        });
    }

    /// Get the allowed methods for a path and subdomain
    /// ## Args
    /// - path: &str
    /// - subdomain: Option<&str>
    /// ## Returns
    /// - Vec<HttpMethod>
    pub fn get_allowed_methods(&self, path: &str, subdomain: Option<&str>) -> Vec<HttpMethod> {
        self.routes
            .iter()
            .filter(|route| {
                (route.subdomain.is_none() || route.subdomain.as_deref() == subdomain)
                    && (route.path == path || self.match_dynamic_path(&route.path, path))
            })
            .map(|route| route.method.clone())
            .collect()
    }

    /// Add a middleware to the Router
    /// ## Args
    /// - middleware: F
    /// ## Where
    /// - F: Fn(&Request) -> Option<Response> + Send + Sync + 'static
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Adds a Middleware to the Router
    pub fn add_middleware<F>(&mut self, middleware: F)
    where
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static,
    {
        self.middlewares.push(Box::new(middleware));
    }

    /// Find a route by path, method, and subdomain
    /// ## Args
    /// - path: &str
    /// - method: &HttpMethod
    /// - subdomain: Option<&str>
    /// ## Returns
    /// - Option<&Route>
    pub fn find_route(&self, path: &str, method: &HttpMethod, subdomain: Option<&str>) -> Option<&Route> {
        self.routes.iter().find(|route| {
            route.method == *method
                && (route.subdomain.as_deref() == subdomain) // Strict comparison of the subdomain
                && (route.path == path || self.match_dynamic_path(&route.path, path))
        })
    }    

    /// Find a route by path and subdomain
    /// ## Args
    /// - path: &str
    /// - subdomain: Option<&str>
    /// ## Returns
    /// - Option<&Route>
    pub fn find_route_by_path_and_subdomain(
        &self,
        path: &str,
        subdomain: Option<&str>,
    ) -> Option<&Route> {
        self.routes.iter().find(|route| {
            route.subdomain.as_deref() == subdomain // Strict comparison of the subdomain
                && (route.path == path || self.match_dynamic_path(&route.path, path))
        })
    }
    
    /// Match a dynamic path
    /// ## Args
    /// - route_path: &str
    /// - request_path: &str
    /// ## Returns
    /// - bool
    fn match_dynamic_path(&self, route_path: &str, request_path: &str) -> bool {
        let route_parts: Vec<&str> = route_path.split('/').collect();
        let request_parts: Vec<&str> = request_path.split('/').collect();

        if route_parts.len() != request_parts.len() {
            return false;
        }

        route_parts.iter().zip(request_parts.iter()).all(|(route_part, request_part)| {
            route_part.starts_with(':') || route_part == request_part
        })
    }

    /// Extract parameters from a path
    /// ## Args
    /// - route_path: &str
    /// - request_path: &str
    /// ## Returns
    /// - std::collections::HashMap<String, String>
    pub fn extract_params(&self, route_path: &str, request_path: &str) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();

        let route_parts: Vec<&str> = route_path.split('/').collect();
        let request_parts: Vec<&str> = request_path.split('/').collect();

        for (route_part, request_part) in route_parts.iter().zip(request_parts.iter()) {
            if route_part.starts_with(':') {
                let key = route_part.trim_start_matches(':').to_string();
                params.insert(key, request_part.to_string());
            }
        }
        params
    }
}