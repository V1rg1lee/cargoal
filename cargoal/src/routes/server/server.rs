use super::super::super::renderer::TemplateRenderer;
use super::super::http::request::parse_request;
use super::super::http::response::format_response;
use super::super::http::HttpMethod;
use super::super::http::Response;
use super::super::routing::GroupBuilder;
use super::super::routing::RouteBuilder;
use super::super::routing::Router;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::fs;

/// Define the Server struct
/// ## Fields
/// - address: String
/// - router: Router
/// - template_dirs: Vec<String>
/// - static_dirs: String
/// - max_static_file_size: usize
pub struct Server {
    address: String,
    pub router: Router,
    pub(crate) template_dirs: Vec<String>,
    pub(crate) static_dirs: String,
    pub(crate) max_static_file_size: usize,
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
            static_dirs: "static".to_string(),
            max_static_file_size: 5 * 1024 * 1024,
        }
    }

    pub fn route<'a>(&'a mut self, path: &'a str, method: HttpMethod) -> RouteBuilder<'a> {
        RouteBuilder::new(path, method, self)
    }

    /// Initialize the Server with the given template directories
    /// ## Args
    /// - dirs: Vec<&str>
    /// ## Returns
    /// - &mut Self
    pub fn with_template_dirs(&mut self, dirs: Vec<&str>) -> &mut Self {
        self.template_dirs = dirs.into_iter().map(String::from).collect();
        self
    }

    /// Add a static directory to the Server
    /// ## Args
    /// - self
    /// - dir: &str
    /// ## Returns
    /// - &mut Self
    pub fn with_static_dir(&mut self, dir: &str) -> &mut Self {
        self.static_dirs = dir.to_string();
        self
    }

    /// Set the maximum file size for static files
    /// ## Args
    /// - self
    /// - size: usize
    /// ## Returns
    /// - &mut Self
    pub fn with_max_static_file_size(&mut self, size: usize) -> &mut Self {
        self.max_static_file_size = size;
        self
    }

    /// Get the TemplateRenderer for the Server
    /// ## Args
    /// - self
    /// ## Returns
    /// - TemplateRenderer
    pub(crate) fn get_template_renderer(&self) -> TemplateRenderer {
        TemplateRenderer::new(self.template_dirs.iter().map(String::as_str).collect())
    }

    /// Detect the MIME type of a file based on its extension
    /// ## Args
    /// - path: &str
    /// ## Returns
    /// - &str
    fn detect_mime_type(path: &Path) -> &str {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("ico") => "image/x-icon",
            Some("svg") => "image/svg+xml",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            Some("otf") => "font/otf",
            Some("json") => "application/json",
            Some("xml") => "application/xml",
            _ => "application/octet-stream",
        }
    }

    /// Check if a file is forbidden
    /// ## Args
    /// - path: &Path
    /// ## Returns
    /// - bool
    fn is_forbidden_file(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            match ext {
                "php" | "exe" | "sh" | "bat" | "cmd" => return true,
                _ => {}
            }
        }
        false
    }

    /// Sanitize a static path
    /// ## Args
    /// - self
    /// - requested_file: &str
    /// ## Returns
    /// - Option<PathBuf>
    fn sanitize_static_path(&self, requested_file: &str) -> Option<PathBuf> {
        if requested_file.contains("..") || requested_file.contains("./") || requested_file.contains(".\\") {
            return None;
        }
        if requested_file.starts_with('/') || requested_file.starts_with('\\') {
            return None;
        }
    
        let root = PathBuf::from(&self.static_dirs).canonicalize().ok()?;
        let mut candidate = root.clone();
        candidate.push(requested_file);
    
        if candidate.is_symlink() {
            return None;
        }
    
        let candidate = candidate.canonicalize().ok()?;
        if candidate.starts_with(&root) {
            Some(candidate)
        } else {
            None
        }
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

        if request.path.starts_with("/static/") {
            let requested_file = &request.path[8..];
            if let Some(safe_path) = self.sanitize_static_path(requested_file) {
                if safe_path.is_dir() || Self::is_forbidden_file(&safe_path) {
                    let response = Response::new(403, Some("Forbidden".to_string()));
                    stream.write(format_response(response).as_bytes()).unwrap();
                    stream.flush().unwrap();
                    return;
                }

                if let Ok(metadata) = fs::metadata(&safe_path) {
                    if metadata.len() > self.max_static_file_size as u64 {
                        let response = Response::new(413, Some("Payload Too Large".to_string()));
                        stream.write(format_response(response).as_bytes()).unwrap();
                        stream.flush().unwrap();
                        return;
                    }
                }
        
                if safe_path.exists() && safe_path.is_file() {
                    match fs::read(&safe_path) {
                        Ok(content) => {
                            let response = Response::new(200, Some(String::from_utf8_lossy(&content).to_string()))
                                .with_header("Content-Type", Self::detect_mime_type(&safe_path))
                                .with_header("X-Content-Type-Options", "nosniff") 
                                .with_header("X-Frame-Options", "DENY");
                            stream.write(format_response(response).as_bytes()).unwrap();
                        }
                        Err(_) => {
                            let response = Response::new(500, Some("Internal Server Error".to_string()));
                            stream.write(format_response(response).as_bytes()).unwrap();
                        }
                    }
                } else {
                    let response = Response::new(404, Some("File Not Found".to_string()));
                    stream.write(format_response(response).as_bytes()).unwrap();
                }
            } else {
                let response = Response::new(403, Some("Forbidden".to_string()));
                stream.write(format_response(response).as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            return;
        }

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
            .or_else(|| {
                // Extract the subdomain from the X-Mock-Subdomain header (for testing)
                request_str
                    .lines()
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
            if self
                .router
                .find_route(&new_path, &request.method, subdomain.as_deref())
                .is_some()
            {
                let redirect_response = Response::new(301, None).with_header("Location", &new_path);
                stream
                    .write(format_response(redirect_response).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
                return;
            }
        }

        println!("Parsed request: {:?} {:?}", request.method, request.path);

        // Search for a matching route
        if let Some(route) =
            self.router
                .find_route(&request.path, &request.method, subdomain.as_deref())
        {
            if let Some(regex) = &route.regex {
                if !regex.is_match(&request.path) {
                    // Check if the path matches the regex, if not return 404
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
            route.subdomain.as_deref() == subdomain.as_deref() && route.path == request.path
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
