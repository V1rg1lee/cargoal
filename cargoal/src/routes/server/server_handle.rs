use crate::renderer::TemplateRenderer;
use crate::routes::http::method::HttpMethod;
use crate::routes::http::request::parse_request;
use crate::routes::http::request::Request;
use crate::routes::http::response::format_response;
use crate::routes::http::response::Response;
use crate::routes::routing::RouteBuilder;
use crate::routes::routing::{GroupBuilder, Router};
use crate::routes::server::core::Server;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

/// Define the ServerHandle struct
/// ## Fields
/// - inner: Arc<Mutex<Server>>
/// - router: Arc<RwLock<Router>>
#[derive(Clone)]
pub struct ServerHandle {
    inner: Arc<Mutex<Server>>,
    router: Arc<RwLock<Router>>,
}

/// Implement the ServerHandle struct
impl ServerHandle {
    /// Create a new ServerHandle instance
    /// ## Args
    /// - address: &str
    /// ## Returns
    /// - ServerHandle
    pub fn new(address: &str) -> Self {
        let router = Arc::new(RwLock::new(Router::new()));
        Self {
            inner: Arc::new(Mutex::new(Server::new(address))),
            router,
        }
    }

    /// Add a middleware to the Server
    /// ## Args
    /// - middleware: F
    /// ## Where
    /// - F: Fn(&Request) -> Option<Response> + Send + Sync + 'static
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Adds a middleware to the Server
    pub async fn add_middleware<F>(&self, middleware: F)
    where
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static,
    {
        let mut router = self.router.write().await;
        router.middlewares.push(Arc::new(middleware));
    }

    /// Add a group of routes to the Server
    /// ## Args
    /// - prefix: &str
    /// - group: F
    /// ## Where
    /// - F: FnOnce(Arc<Mutex<GroupBuilder>>) -> Fut + Send + 'static
    /// - Fut: std::future::Future<Output = ()> + Send
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Adds a group of routes to the Server
    pub async fn with_group<F, Fut>(&self, prefix: &str, group: F)
    where
        F: FnOnce(Arc<Mutex<GroupBuilder>>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let group_builder = Arc::new(Mutex::new(GroupBuilder::new(prefix, self.clone())));

        tokio::spawn(async move {
            group(Arc::clone(&group_builder)).await;
        })
        .await
        .unwrap();
    }

    /// Add a route to the Server
    /// ## Args
    /// - path: &str
    /// - method: HttpMethod
    /// ## Returns
    /// - RouteBuilder
    pub fn route(&self, path: &str, method: HttpMethod) -> RouteBuilder {
        RouteBuilder::new(path, method, self.clone())
    }

    /// Get the router of the Server
    /// ## Returns
    /// - Arc<RwLock<Router>>
    pub(crate) fn router(&self) -> Arc<RwLock<Router>> {
        Arc::clone(&self.router)
    }

    /// Get the template renderer of the Server
    /// ## Returns
    /// - TemplateRenderer
    pub(crate) async fn get_template_renderer(&self) -> TemplateRenderer {
        let server = self.inner.lock().await;
        TemplateRenderer::new(server.template_dirs.iter().map(String::as_str).collect())
    }

    /// Get the address of the server
    /// ## Returns
    /// - String
    pub(crate) async fn address(&self) -> String {
        let server = self.inner.lock().await;
        server.address.clone()
    }

    /// Set the maximum static file size
    /// ## Args
    /// - size: usize
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Sets the maximum static file size
    pub async fn with_max_static_file_size(&mut self, size: usize) {
        let mut server = self.inner.lock().await;
        server.max_static_file_size = size;
    }

    /// Set the static directory
    /// ## Args
    /// - dir: &str
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Sets the static directory
    pub async fn with_static_dir(&mut self, dir: &str) {
        let mut server = self.inner.lock().await;
        server.static_dirs = dir.to_string();
    }

    /// Set the template directories
    /// ## Args
    /// - dirs: Vec<&str>
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Sets the template directories
    pub async fn with_template_dirs(&mut self, dirs: Vec<&str>) {
        let mut server = self.inner.lock().await;
        server.template_dirs = dirs.iter().map(|d| d.to_string()).collect();
    }

    /// Run the server
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Runs the server
    pub async fn run(&self) {
        let listener = TcpListener::bind(self.address().await).await.unwrap();
        println!("Server running on {}", self.address().await);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let handle_clone = self.clone();
                    tokio::spawn(async move {
                        handle_clone.handle_connection(stream).await;
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }
    }

    /// Send a response to a client
    /// ## Args
    /// - stream: &mut TcpStream
    /// - response: Response
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Sends a response to a client
    async fn send_response(stream: &mut TcpStream, response: Response) {
        let response_str = format_response(response);
        if let Err(e) = stream.write_all(response_str.as_bytes()).await {
            eprintln!("Error writing response: {}", e);
        }
        if let Err(e) = stream.flush().await {
            eprintln!("Error flushing stream: {}", e);
        }
    }

    /// Check if a file is forbidden
    /// ## Args
    /// - path: &Path
    /// ## Returns
    /// - bool
    fn is_forbidden_file(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("php" | "exe" | "sh" | "bat" | "cmd")
        )
    }

    /// Sanitize a static file path to prevent directory traversal attacks
    /// ## Args
    /// - requested_file: &str
    /// ## Returns
    /// - Option<PathBuf>
    async fn sanitize_static_path(&self, requested_file: &str) -> Option<PathBuf> {
        let server = self.inner.lock().await;
        let static_dirs = &server.static_dirs;

        if requested_file.contains("..")
            || requested_file.contains("./")
            || requested_file.contains(".\\")
        {
            return None;
        }
        if requested_file.starts_with('/') || requested_file.starts_with('\\') {
            return None;
        }

        let root = fs::canonicalize(static_dirs).await.ok()?;
        let mut candidate = root.clone();
        candidate.push(requested_file);

        if fs::symlink_metadata(&candidate)
            .await
            .ok()?
            .file_type()
            .is_symlink()
        {
            return None;
        }

        let candidate = fs::canonicalize(&candidate).await.ok()?;
        if candidate.starts_with(&root) {
            Some(candidate)
        } else {
            None
        }
    }

    /// Detect the MIME type of a file based on its extension
    /// ## Args
    /// - path: &Path
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

    /// Handle a connection
    /// ## Args
    /// - stream: TcpStream
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Handles a connection
    async fn handle_connection(&self, mut stream: TcpStream) {
        let router = self.router();

        let mut buffer = Vec::new();
        let mut temp_buffer = [0; 1024];

        // read the request
        loop {
            match stream.read(&mut temp_buffer).await {
                Ok(0) => return, // EOF
                Ok(n) => {
                    buffer.extend_from_slice(&temp_buffer[..n]);
                    if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading request: {}", e);
                    return;
                }
            }
        }

        let request_str = String::from_utf8_lossy(&buffer[..]);
        println!("Received request:\n{}", request_str);

        let mut request = parse_request(&request_str);

        let router_lock = router.read().await;

        // Execute global middlewares
        for middleware in &router_lock.middlewares {
            if let Some(response) = middleware(&request) {
                Self::send_response(&mut stream, response).await;
                return;
            }
        }

        // verify if the request is for a static file
        if request.path.starts_with("/static/") {
            let requested_file = &request.path[8..];
            if let Some(safe_path) = self.sanitize_static_path(requested_file).await {
                if safe_path.is_dir() || Self::is_forbidden_file(&safe_path) {
                    Self::send_response(
                        &mut stream,
                        Response::new(403, Some("Forbidden".to_string())),
                    )
                    .await;
                    return;
                }

                match fs::metadata(&safe_path).await {
                    Ok(metadata)
                        if metadata.len() > self.inner.lock().await.max_static_file_size as u64 =>
                    {
                        Self::send_response(
                            &mut stream,
                            Response::new(413, Some("Payload Too Large".to_string())),
                        )
                        .await;
                        return;
                    }
                    Ok(_) => match fs::read(&safe_path).await {
                        Ok(content) => {
                            let response = Response::new(
                                200,
                                Some(String::from_utf8_lossy(&content).to_string()),
                            )
                            .with_header("Content-Type", Self::detect_mime_type(&safe_path))
                            .with_header("X-Content-Type-Options", "nosniff")
                            .with_header("X-Frame-Options", "DENY");
                            Self::send_response(&mut stream, response).await;
                        }
                        Err(_) => {
                            Self::send_response(
                                &mut stream,
                                Response::new(500, Some("Internal Server Error".to_string())),
                            )
                            .await;
                        }
                    },
                    Err(_) => {
                        Self::send_response(
                            &mut stream,
                            Response::new(404, Some("File Not Found".to_string())),
                        )
                        .await;
                    }
                }
                return;
            } else {
                Self::send_response(
                    &mut stream,
                    Response::new(403, Some("Forbidden".to_string())),
                )
                .await;
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

        // Redirect if the path ends with a trailing slash
        if request.path.ends_with('/') && request.path != "/" {
            let new_path = request.path.trim_end_matches('/').to_string();
            if router_lock
                .find_route(&new_path, &request.method, subdomain.as_deref())
                .is_some()
            {
                let redirect_response = Response::new(301, None).with_header("Location", &new_path);
                Self::send_response(&mut stream, redirect_response).await;
                return;
            }
        }

        println!("Parsed request: {:?} {:?}", request.method, request.path);

        // Search for a matching route
        if let Some(route) =
            router_lock.find_route(&request.path, &request.method, subdomain.as_deref())
        {
            if let Some(regex) = &route.regex {
                if !regex.is_match(&request.path) {
                    Self::send_response(
                        &mut stream,
                        Response::new(404, Some("Not Found".to_string())),
                    )
                    .await;
                    return;
                }
            }

            // Execute route middlewares
            for middleware in &route.middlewares {
                if let Some(response) = middleware(&request) {
                    Self::send_response(&mut stream, response).await;
                    return;
                }
            }

            // Extract route parameters
            let route_params = router_lock.extract_params(route, &request.path);
            request.params.extend(route_params);

            // Execute the route handler
            let response = (route.handler)(request);
            Self::send_response(&mut stream, response).await;
        } else {
            // Check if the path is allowed but the method is not
            if router_lock
                .routes
                .iter()
                .any(|r| r.path == request.path && r.subdomain.as_deref() == subdomain.as_deref())
            {
                let allowed_methods =
                    router_lock.get_allowed_methods(&request.path, subdomain.as_deref());
                let allow_header = allowed_methods
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let response = Response::new(405, Some("Method Not Allowed".to_string()))
                    .with_header("Allow", &allow_header);
                Self::send_response(&mut stream, response).await;
            } else {
                Self::send_response(
                    &mut stream,
                    Response::new(404, Some("Not Found".to_string())),
                )
                .await;
            }
        }
    }
}
