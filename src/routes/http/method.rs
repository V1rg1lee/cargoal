use std::fmt;

/// Define the HttpMethod enum
///  - GET
///  - POST
///  - PUT
///  - DELETE
///  - PATCH
///  - OPTIONS
///  - HEAD
///  - TRACE
///  - CONNECT
///  - OTHER(String)
#[derive(PartialEq, Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    TRACE,
    CONNECT,
    OTHER(String),
}

/// Implement the HttpMethod enumq
impl HttpMethod {
    /// Convert a string to a HttpMethod
    /// ## Args
    /// - method: &str
    /// ## Returns
    /// - HttpMethod 
    pub(crate) fn from_str(method: &str) -> Self {
        match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "OPTIONS" => HttpMethod::OPTIONS,
            "HEAD" => HttpMethod::HEAD,
            "TRACE" => HttpMethod::TRACE,
            "CONNECT" => HttpMethod::CONNECT,
            other => HttpMethod::OTHER(other.to_string()),
        }
    }
}

/// Implement the Display trait for HttpMethod
impl fmt::Display for HttpMethod {
    /// Format the HttpMethod
    /// ## Args
    /// - self
    /// - f: &mut fmt::Formatter<'_>
    /// ## Returns
    /// - fmt::Result
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method_str = match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::OTHER(other) => other,
        };
        write!(f, "{}", method_str)
    }
}