use super::HttpMethod;

/// Define the Request struct
/// ## Fields
/// - path: String
/// - method: HttpMethod
/// - body: Option<String>
/// - params: std::collections::HashMap<String, String>
pub struct Request {
    pub path: String,
    pub method: HttpMethod,
    pub body: Option<String>,
    pub params: std::collections::HashMap<String, String>,
}

/// Parse a raw HTTP request string into a Request struct
/// ## Args
/// - request: &str
/// ## Returns
/// - Request
pub(crate) fn parse_request(request: &str) -> Request {
    let lines: Vec<&str> = request.lines().collect();
    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    // Extract the path and query parameters
    let (path, query) = if let Some(pos) = parts[1].find('?') {
        (&parts[1][..pos], Some(&parts[1][pos + 1..]))
    } else {
        (parts[1], None)
    };

    // Parse the query parameters
    let query_params = query
        .map(|q| {
            q.split('&')
                .filter_map(|pair| {
                    let mut kv = pair.split('=');
                    if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                        Some((key.to_string(), value.to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    println!("Parsed path: {}", path);
    println!("Parsed query params: {:?}", query_params);

    // Extract the body of the request
    let body = if lines.len() > 1 {
        Some(lines[lines.len() - 1].to_string())
    } else {
        None
    };

    Request {
        path: path.to_string(),
        method: HttpMethod::from_str(parts[0]),
        body,
        params: query_params,
    }
}
