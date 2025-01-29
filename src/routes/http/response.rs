/// Define the Response struct
/// ## Fields
/// - status_code: u16
/// - headers: std::collections::HashMap<String, String>
/// - body: Option<String>
pub struct Response {
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
}

impl Response {
    /// Create a new Response
    /// ## Args
    /// - status_code: u16
    /// - body: Option<String>
    /// ## Returns
    /// - Response
    pub fn new(status_code: u16, body: Option<String>) -> Self {
        Self {
            status_code,
            headers: std::collections::HashMap::new(),
            body,
        }
    }

    /// Add a header to the Response
    /// ## Args
    /// - self
    /// - key: &str
    /// - value: &str
    /// ## Returns
    /// - Response
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

/// Format a Response into a raw HTTP response string
/// ## Args
/// - response: Response
/// ## Returns
/// - String
pub(crate) fn format_response(response: Response) -> String {
    let mut response_str = format!("HTTP/1.1 {} OK\r\n", response.status_code);

    for (key, value) in response.headers {
        response_str.push_str(&format!("{}: {}\r\n", key, value));
    }

    response_str.push_str("\r\n");

    if let Some(body) = response.body {
        response_str.push_str(&body);
    }

    response_str
}