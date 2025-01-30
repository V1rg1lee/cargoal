use super::super::http::HttpMethod;
use super::super::http::Request;
use super::super::http::Response;
use super::middleware::Middleware;
use super::Route;
use regex::Regex;
use std::sync::Arc;

/// Define the Router struct
/// ## Fields
/// - routes: Vec<Route>
/// - middlewares: Vec<Middleware>
pub struct Router {
    pub(crate) routes: Vec<Route>,
    pub(crate) middlewares: Vec<Middleware>,
}

/// Implement the Router struct
impl Router {
    /// Create a new Router instance
    /// ## Returns
    /// - Router
    pub(crate) fn new() -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
        }
    }

    /// Add a route to the Router
    /// ## Args
    /// - subdomain: Option<&str>
    /// - path: &str
    /// - method: HttpMethod
    /// - handler: F
    /// - regex: Option<&str>
    /// ## Where
    /// - F: Fn(Request) -> Response + Send + Sync + 'static
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Adds a Route to the Router
    pub(crate) fn add_route<F>(
        &mut self,
        subdomain: Option<&str>,
        path: &str,
        method: HttpMethod,
        handler: F,
        regex: Option<&str>,
    ) where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        // Compile the regex if it exists
        let compiled_regex = regex.map(|r| Regex::new(r).unwrap());

        // Compile the dynamic regex if it exists
        let dynamic_regex = regex.or_else(|| {
            if path.contains(":") {
                let mut regex_path = path.to_string();
                regex_path = regex_path.replace(":", "(?P<");
                regex_path = regex_path.replace("/", r"\/");
                regex_path.push_str(">[^/]+)");
                let regex_string = format!("^{}$", regex_path);
                Some(Box::leak(regex_string.into_boxed_str()))
            } else {
                None
            }
        });
        let compiled_dynamic_regex = dynamic_regex.map(|r| Regex::new(&r).unwrap());

        self.routes.push(Route {
            subdomain: subdomain.map(|s| s.to_string()),
            path: path.to_string(),
            method,
            handler: Box::new(handler),
            regex: compiled_regex.or(compiled_dynamic_regex),
            middlewares: Vec::new(),
        });
    }

    /// Get the allowed methods for a path and subdomain
    /// ## Args
    /// - path: &str
    /// - subdomain: Option<&str>
    /// ## Returns
    /// - Vec<HttpMethod>
    pub(crate) fn get_allowed_methods(
        &self,
        path: &str,
        subdomain: Option<&str>,
    ) -> Vec<HttpMethod> {
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
        self.middlewares.push(Arc::new(middleware));
    }

    /// Find a route by path, method, and subdomain
    /// ## Args
    /// - path: &str
    /// - method: &HttpMethod
    /// - subdomain: Option<&str>
    /// ## Returns
    /// - Option<&Route>
    pub(crate) fn find_route(
        &self,
        path: &str,
        method: &HttpMethod,
        subdomain: Option<&str>,
    ) -> Option<&Route> {
        self.routes.iter().find(|route| {
            route.method == *method
                && (route.subdomain.as_deref() == subdomain)
                && (route.path == path
                    || route.regex.as_ref().map_or(false, |re| re.is_match(path)) // Correspondance regex
                    || self.match_dynamic_path(&route.path, path))
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

        route_parts
            .iter()
            .zip(request_parts.iter())
            .all(|(route_part, request_part)| {
                route_part.starts_with(':') || route_part == request_part
            })
    }

    /// Extract parameters from a path
    /// ## Args
    /// - route: &Route
    /// - request_path: &str
    /// ## Returns
    /// - std::collections::HashMap<String, String>
    pub(crate) fn extract_params(
        &self,
        route: &Route,
        request_path: &str,
    ) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();

        if let Some(regex) = &route.regex {
            if let Some(captures) = regex.captures(request_path) {
                for name in regex.capture_names().flatten() {
                    if let Some(value) = captures.name(name) {
                        params.insert(name.to_string(), value.as_str().to_string());
                    }
                }
            }
        } else {
            let route_parts: Vec<&str> = route.path.split('/').collect();
            let request_parts: Vec<&str> = request_path.split('/').collect();

            for (route_part, request_part) in route_parts.iter().zip(request_parts.iter()) {
                if route_part.starts_with(':') {
                    let key = route_part.trim_start_matches(':').to_string();
                    params.insert(key, request_part.to_string());
                }
            }
        }

        params
    }
}
