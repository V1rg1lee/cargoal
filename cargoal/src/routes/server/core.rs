/// Define the Server struct
/// ## Fields
/// - address: String
/// - template_dirs: Vec<String>
/// - static_dirs: String
/// - max_static_file_size: usize
pub(crate) struct Server {
    pub(crate) address: String,
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
            template_dirs: vec!["templates".to_string()],
            static_dirs: "static".to_string(),
            max_static_file_size: 5 * 1024 * 1024,
        }
    }
}