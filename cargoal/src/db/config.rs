use std::env;

/// Define the DatabaseType enum
///
/// ## Variants
/// - Postgres
/// - MySql (same as MariaDB)
/// - Sqlite
#[derive(Debug)]
pub enum DatabaseType {
    Postgres,
    MySql,
    Sqlite,
}

impl DatabaseType {
    /// Convert a string to a DatabaseType
    ///
    /// ## Args
    /// - s: &str
    ///
    /// ## Returns
    /// - Self if the string is a valid DatabaseType
    /// - None otherwise
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "postgres" => Some(DatabaseType::Postgres),
            "mysql" => Some(DatabaseType::MySql),
            "sqlite" => Some(DatabaseType::Sqlite),
            _ => None,
        }
    }
}

/// Define the DbConfig struct
///
/// ## Fields
/// - db_type: DatabaseType
/// - database_url: String
/// - max_connections: Option<u32>
/// - timeout_seconds: Option<u64>
///
/// ## Methods
/// - new: Create a new DbConfig
/// - from_env: Create a new DbConfig from environment variables
#[derive(Debug)]
pub struct DbConfig {
    pub db_type: DatabaseType,
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

impl DbConfig {
    /// Create a new DbConfig
    ///
    /// ## Args
    /// - db_type: DatabaseType
    /// - database_url: String
    /// - max_connections: Option<u32>
    /// - timeout_seconds: Option<u64>
    ///
    /// ## Returns
    /// - Self
    pub fn new(
        db_type: DatabaseType,
        database_url: String,
        max_connections: Option<u32>,
        timeout_seconds: Option<u64>,
    ) -> Self {
        Self {
            db_type,
            database_url,
            max_connections,
            timeout_seconds,
        }
    }

    /// Create a new DbConfig from environment variables
    ///
    /// ## Returns
    /// - Self
    pub fn from_env() -> Self {
        let db_type = match env::var("DATABASE_TYPE")
            .unwrap_or_else(|_| "postgres".to_string())
            .as_str()
        {
            "postgres" => DatabaseType::Postgres,
            "mysql" => DatabaseType::MySql,
            "sqlite" => DatabaseType::Sqlite,
            _ => panic!("DATABASE_TYPE invalide !"),
        };

        Self {
            db_type,
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL doit être défini"),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok()),
            timeout_seconds: env::var("DATABASE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok()),
        }
    }
}
