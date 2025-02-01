use std::env;
use std::str::FromStr;
use std::sync::OnceLock;
use std::sync::RwLock;

static DATABASE_TYPE: OnceLock<RwLock<DatabaseType>> = OnceLock::new();

/// Define the DatabaseType enum
///
/// ## Variants
/// - Postgres
/// - MySql (same as MariaDB)
/// - Sqlite
#[derive(Debug, Clone, Copy)]
pub enum DatabaseType {
    Postgres,
    MySql,
    Sqlite,
}

impl FromStr for DatabaseType {
    type Err = ();

    /// Convert a string to a DatabaseType
    ///
    /// ## Args
    /// - s: &str
    ///
    /// ## Returns
    /// - Result<Self, Self::Err>
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "postgres" => Ok(DatabaseType::Postgres),
            "mysql" => Ok(DatabaseType::MySql),
            "sqlite" => Ok(DatabaseType::Sqlite),
            _ => Err(()),
        }
    }
}

impl DatabaseType {
    /// Get the current database type, set by the user
    ///
    /// ## Returns
    /// - Option<DatabaseType>
    fn get_db_type() -> Option<DatabaseType> {
        DATABASE_TYPE
            .get()
            .and_then(|db| db.read().ok().map(|db_type| *db_type))
    }

    /// Set the database type
    ///
    /// ## Args
    /// - db_type: DatabaseType
    ///
    /// ## Example
    /// ```rust,ignore
    /// use cargoal::db::config::{DatabaseType};
    ///
    /// DatabaseType::set_db_type(DatabaseType::Postgres);
    /// ```
    pub(crate) fn set_db_type(db_type: DatabaseType) {
        DATABASE_TYPE.get_or_init(|| RwLock::new(db_type));
    }

    /// Map Rust types to SQL types based on the database type
    ///
    /// ## Args
    /// - rust_type: &str
    ///
    /// ## Returns
    /// - (String, bool)
    ///
    /// ## Example
    /// ```rust,ignore
    /// use cargoal::db::config::{DatabaseType};
    ///
    /// let (sql_type, optional) = DatabaseType::rust_type_to_sql_type("i32");
    ///
    /// assert_eq!(sql_type, "INTEGER");
    /// assert_eq!(optional, false);
    /// ```
    pub fn rust_type_to_sql_type(rust_type: &str) -> (String, bool) {
        let (base_sql_type, is_nullable) = match rust_type
            .strip_prefix("Option<")
            .and_then(|t| t.strip_suffix('>'))
        {
            Some(inner_type) => (Self::rust_type_to_generic_sql(inner_type), true),
            None => (Self::rust_type_to_generic_sql(rust_type), false),
        };

        let adapted_sql_type = match Self::get_db_type().expect("Database type not set") {
            Self::Postgres | Self::Sqlite => base_sql_type.to_string(),
            Self::MySql => match base_sql_type {
                "INTEGER" => "INT".to_string(),
                "BOOLEAN" => "TINYINT(1)".to_string(),
                "TEXT" => "VARCHAR(255)".to_string(),
                _ => base_sql_type.to_string(),
            },
        };

        (adapted_sql_type, is_nullable)
    }

    /// Map Rust types to generic SQL types
    ///
    /// ## Args
    /// - rust_type: &str
    ///
    /// ## Returns
    /// - &str
    fn rust_type_to_generic_sql(rust_type: &str) -> &str {
        match rust_type {
            "i32" | "u32" | "i64" | "u64" => "INTEGER",
            "String" => "TEXT",
            "f32" | "f64" => "REAL",
            "bool" => "BOOLEAN",
            _ => "TEXT",
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
