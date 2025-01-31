use super::super::super::renderer::TemplateRenderer;
use super::super::http::request::parse_request;
use super::super::http::response::format_response;
use super::super::http::HttpMethod;
use super::super::http::Response;
use super::super::routing::GroupBuilder;
use super::super::routing::RouteBuilder;
use super::super::routing::Router;
use super::super::http::Request;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Define the Server struct
/// ## Fields
/// - address: String
/// - router: Arc<Mutex<Router>>
/// - template_dirs: Vec<String>
/// - static_dirs: String
/// - max_static_file_size: usize
pub struct Server {
    address: String,
    pub router: Arc<Mutex<Router>>,
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
            router: Arc::new(Mutex::new(Router::new())),
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
        matches!(path.extension().and_then(|ext| ext.to_str()), Some("php" | "exe" | "sh" | "bat" | "cmd"))
    }

    /// Sanitize a static path
    /// ## Args
    /// - requested_file: &str
    /// - static_dirs: &str
    /// ## Returns
    /// - Option<PathBuf>
    async fn sanitize_static_path(requested_file: &str, static_dirs: &str) -> Option<PathBuf> {
        if requested_file.contains("..") || requested_file.contains("./") || requested_file.contains(".\\") {
            return None;
        }
        if requested_file.starts_with('/') || requested_file.starts_with('\\') {
            return None;
        }

        let root = fs::canonicalize(static_dirs).await.ok()?;
        let mut candidate = root.clone();
        candidate.push(requested_file);

        if fs::symlink_metadata(&candidate).await.ok()?.file_type().is_symlink() {
            return None;
        }

        let candidate = fs::canonicalize(&candidate).await.ok()?;
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
    pub async fn run(&self) {
        let listener = TcpListener::bind(&self.address).await.unwrap();
        println!("Server running on {}", self.address);

        let router = Arc::clone(&self.router);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let router = Arc::clone(&router);
                    let static_dirs = self.static_dirs.clone();
                    let max_static_file_size = self.max_static_file_size;
                    tokio::spawn(async move {
                        Self::handle_connection(stream, router, static_dirs, max_static_file_size).await;
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }
    }

    /// Send a response to a stream
    /// ## Args
    /// - stream: &mut TcpStream
    /// - response: Response
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Writes to the stream
    async fn send_response(stream: &mut TcpStream, response: Response) {
        let response_str = format_response(response);

        if let Err(e) = stream.write_all(response_str.as_bytes()).await {
            eprintln!("Error writing response: {}", e);
        }

        if let Err(e) = stream.flush().await {
            eprintln!("Error flushing stream: {}", e);
        }
    }

    /// Add a middleware to the server
    /// ## Args
    /// - self
    /// - middleware: F
    /// ## Where
    /// - F: Fn(&Request) -> Option<Response> + Send + Sync + 'static
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Adds a middleware to the server
    pub async fn add_middleware<F>(&self, middleware: F)
    where
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static,
    {
        let mut router = self.router.lock().await;
        router.middlewares.push(Arc::new(middleware));
    }

    /// Handle a connection
    /// ## Args
    /// - stream: TcpStream
    /// - router: Arc<Mutex<Router>>
    /// - static_dirs: String
    /// - max_static_file_size: usize
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Reads and writes to the stream
    async fn handle_connection(mut stream: TcpStream, router: Arc<Mutex<Router>>, static_dirs: String, max_static_file_size: usize) {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0; 1024];

        loop {
            match stream.read(&mut temp_buffer).await {
                Ok(0) => return,
                Ok(n) => {
                    buffer.extend_from_slice(&temp_buffer[..n]);
                    if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Erreur de lecture de la requête: {}", e);
                    return;
                }
            }
        }

        let request_str = String::from_utf8_lossy(&buffer[..]);
        println!("Request received:\n{}", request_str);

        let mut request = parse_request(&request_str);

        if request.path.starts_with("/static/") {
            let requested_file = &request.path[8..];
            if let Some(safe_path) = Self::sanitize_static_path(requested_file, &static_dirs).await {
                if safe_path.is_dir() || Self::is_forbidden_file(&safe_path) {
                    Self::send_response(&mut stream, Response::new(403, Some("Forbidden".to_string()))).await;
                    return;
                }

                match fs::metadata(&safe_path).await {
                    Ok(metadata) if metadata.len() > max_static_file_size as u64 => {
                        Self::send_response(&mut stream, Response::new(413, Some("Payload Too Large".to_string()))).await;
                        return;
                    }
                    Ok(_) => {
                        match fs::read(&safe_path).await {
                            Ok(content) => {
                                let response = Response::new(200, Some(String::from_utf8_lossy(&content).to_string()))
                                    .with_header("Content-Type", Self::detect_mime_type(&safe_path))
                                    .with_header("X-Content-Type-Options", "nosniff")
                                    .with_header("X-Frame-Options", "DENY");
                                Self::send_response(&mut stream, response).await;
                            }
                            Err(_) => {
                                Self::send_response(&mut stream, Response::new(500, Some("Internal Server Error".to_string()))).await;
                            }
                        }
                    }
                    Err(_) => {
                        Self::send_response(&mut stream, Response::new(404, Some("File Not Found".to_string()))).await;
                    }
                }
                return;
            } else {
                Self::send_response(&mut stream, Response::new(403, Some("Forbidden".to_string()))).await;
                return;
            }
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

        let router = router.lock().await;

        // Middleware execution
        for middleware in &router.middlewares {
            if let Some(response) = middleware(&request) {
                Self::send_response(&mut stream, response).await;
                return;
            }
        }

        if request.path.ends_with('/') && request.path != "/" {
            let new_path = request.path.trim_end_matches('/').to_string();
            if router.find_route(&new_path, &request.method, subdomain.as_deref()).is_some() {
                let redirect_response = Response::new(301, None).with_header("Location", &new_path);
                Self::send_response(&mut stream, redirect_response).await;
                return;
            }
        }

        println!("Parsed request: {:?} {:?}", request.method, request.path);

        // Search for a matching route
        if let Some(route) = router.find_route(&request.path, &request.method, subdomain.as_deref()) {
            if let Some(regex) = &route.regex {
                if !regex.is_match(&request.path) {
                    Self::send_response(&mut stream, Response::new(404, Some("Not Found".to_string()))).await;
                    return;
                }
            }

            // Exécution des middlewares spécifiques à la route
            for middleware in &route.middlewares {
                if let Some(response) = middleware(&request) {
                    Self::send_response(&mut stream, response).await;
                    return;
                }
            }

            // Add route parameters to the request
            let route_params = router.extract_params(route, &request.path);
            request.params.extend(route_params);

            let response = (route.handler)(request);
            Self::send_response(&mut stream, response).await;
        } else {
            // Check if the path is allowed but the method is not
            if router.routes.iter().any(|r| r.subdomain.as_deref() == subdomain.as_deref() && r.path == request.path) {
                let allowed_methods = router.get_allowed_methods(&request.path, subdomain.as_deref());
                let allow_header = allowed_methods.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ");
                let response = Response::new(405, Some("Method Not Allowed".to_string())).with_header("Allow", &allow_header);
                Self::send_response(&mut stream, response).await;
            } else {
                Self::send_response(&mut stream, Response::new(404, Some("Not Found".to_string()))).await;
            }
        }
    }
}