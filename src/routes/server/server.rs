use super::super::routing::Router;
use super::super::http::response::Response;
use super::super::http::method::HttpMethod;
use super::super::http::request::parse_request;
use super::super::http::response::format_response;
use super::super::super::renderer::renderer::TemplateRenderer;
use super::super::routing::route_builder::RouteBuilder;
use super::super::routing::group_builder::GroupBuilder;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

/// Define the Server struct
/// ## Fields
/// - address: String
/// - router: Router
/// - template_dirs: Vec<String>
pub struct Server {
    address: String,
    pub router: Router,
    pub template_dirs: Vec<String>,
}

/// Implement the Server struct
impl Server {
    /// Create a new Server instance
    /// ## Args
    /// - address: &str
    /// ## Returns
    /// - Server
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            router: Router::new(),
            template_dirs: vec!["templates".to_string()],
        }
    }

    pub fn route<'a>(&'a mut self, path: &'a str, method: HttpMethod) -> RouteBuilder<'a> {
        RouteBuilder::new(path, method, self)
    }

    /// Initialize the Server with the given template directories
    /// ## Args
    /// - dirs: Vec<&str>
    /// ## Returns
    /// - Server
    pub fn with_template_dirs(mut self, dirs: Vec<&str>) -> Self {
        self.template_dirs = dirs.into_iter().map(String::from).collect();
        self
    }

    /// Get the TemplateRenderer for the Server
    /// ## Args
    /// - self
    /// ## Returns
    /// - TemplateRenderer
    pub fn get_template_renderer(&self) -> TemplateRenderer {
        TemplateRenderer::new(self.template_dirs.iter().map(String::as_str).collect())
    }

    /// Add a group of routes to the server
    /// ## Args
    /// - self
    /// - prefix: &str
    /// - group: F
    /// ## Where
    /// - F: FnOnce(&mut GroupBuilder)
    /// ## Returns
    /// - &mut Self
    pub fn with_group<F>(&mut self, prefix: &str, group: F) -> &mut Self
    where
        F: FnOnce(&mut GroupBuilder),
    {
        let mut group_builder = GroupBuilder::new(prefix, self);
        group(&mut group_builder);
        self
    }

    /// Run the server
    /// ## Args
    /// - self
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Starts the server
    pub fn run(&self) {
        let listener = TcpListener::bind(&self.address).unwrap();
        println!("Server running on {}", self.address);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }

    /// Handle a connection
    /// ## Args
    /// - self
    /// - stream: TcpStream
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Reads and writes to the stream
    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
    
        let request_str = String::from_utf8_lossy(&buffer[..]);
        println!("Request received:\n{}", request_str);
    
        let mut request = parse_request(&request_str);
    
        // Extract the subdomain from the Host header
        let subdomain = request_str
            .lines()
            .find(|line| line.to_lowercase().starts_with("host:"))
            .and_then(|host_line| {
                let host = host_line.split_whitespace().nth(1)?;
                if host.contains('.') {
                    let subdomain = host.split('.').next();
                    subdomain.map(String::from)
                } else {
                    None
                }
            })
            .or_else(|| { // Extract the subdomain from the X-Mock-Subdomain header (for testing)
                request_str.lines()
                           .find(|line| line.to_lowercase().starts_with("x-mock-subdomain:"))
                           .and_then(|line| line.split_whitespace().nth(1).map(String::from))
            });
    
        println!("Subdomain: {:?}", subdomain);
    
        // Middleware execution
        for middleware in &self.router.middlewares {
            if let Some(response) = middleware(&request) {
                stream.write(format_response(response).as_bytes()).unwrap();
                stream.flush().unwrap();
                return;
            }
        }
    
        if request.path.ends_with('/') && request.path != "/" {
            let new_path = request.path.trim_end_matches('/').to_string();
        
            // Verify if a route exists for the new path
            if self.router.find_route(&new_path, &request.method, subdomain.as_deref()).is_some() {
                let redirect_response = Response::new(301, None)
                    .with_header("Location", &new_path);
                stream.write(format_response(redirect_response).as_bytes()).unwrap();
                stream.flush().unwrap();
                return;
            }
        }
    
        println!("Parsed request: {:?} {:?}", request.method, request.path);
    
        // Search for a matching route
        if let Some(route) = self.router.find_route(&request.path, &request.method, subdomain.as_deref()) {
            if let Some(regex) = &route.regex {
                if !regex.is_match(&request.path) { // Check if the path matches the regex, if not return 404
                    let response = Response::new(404, Some("Not Found".to_string()));
                    stream.write(format_response(response).as_bytes()).unwrap();
                    stream.flush().unwrap();
                    return;
                }
            }

            // Middleware execution
            for middleware in &route.middlewares {
                if let Some(response) = middleware(&request) {
                    stream.write(format_response(response).as_bytes()).unwrap();
                    stream.flush().unwrap();
                    return;
                }
            }

            // Add the route params to the request
            let route_params = self.router.extract_params(route, &request.path);
            request.params.extend(route_params);
    
            let response = (route.handler)(request);
            stream.write(format_response(response).as_bytes()).unwrap();
        } else if self.router.routes.iter().any(|route| {
            route.subdomain.as_deref() == subdomain.as_deref()
                && route.path == request.path
        }) {
            let allowed_methods = self
                .router
                .get_allowed_methods(&request.path, subdomain.as_deref());
            let allow_header = allowed_methods
                .iter()
                .map(|method| method.to_string())
                .collect::<Vec<_>>()
                .join(", ");
        
            let response = Response::new(405, Some("Method Not Allowed".to_string()))
                .with_header("Allow", &allow_header);
            stream.write(format_response(response).as_bytes()).unwrap();
        } else {
            let response = Response::new(404, Some("Not Found".to_string()));
            stream.write(format_response(response).as_bytes()).unwrap();
        }
    
        stream.flush().unwrap();
    }        
}
