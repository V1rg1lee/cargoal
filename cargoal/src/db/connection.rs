use sqlx::{Error, MySql, Pool, Postgres, Row, Sqlite};
use std::{collections::HashMap, sync::Arc};

use super::config::{DatabaseType, DbConfig};

/// Trait to define an Entity with the following information:
/// - TABLE_NAME: The name of the table
/// - COLUMNS: The columns of the table
/// - TYPES: The data types of the columns
/// - primary_keys: The primary keys of the table
pub trait EntityTrait {
    const TABLE_NAME: &'static str;
    const COLUMNS: &'static [&'static str];
    const TYPES: &'static [&'static str];

    fn primary_keys() -> Vec<&'static str>;
}

/// A type alias for a vector of strings which represents a column information
type ColumnInfos = Vec<(String, String, bool)>; // (column_name, data_type, is_nullable)
/// A type alias for a database table
type DatabaseTable = (String, ColumnInfos);

/// Define the DatabasePool enum
///
/// ## Variants
/// - Postgres
/// - MySql (same as MariaDB)
/// - Sqlite
#[derive(Debug)]
enum DatabasePool {
    Postgres(Pool<Postgres>),
    MySql(Pool<MySql>),
    Sqlite(Pool<Sqlite>),
}

/// Define the EntityMetadata struct
///
/// ## Fields
/// - table_name: String
/// - columns: ColumnInfos
///
///
#[derive(Debug, Clone)]
pub struct EntityMetadata {
    pub table_name: String,
    pub columns: ColumnInfos,
    pub primary_keys: Vec<String>,
}

impl EntityMetadata {
    /// Create a new EntityMetadata from a DatabaseTable
    /// 
    /// ## Args
    /// - table: DatabaseTable
    /// 
    /// ## Returns
    /// - Self
    pub fn from_database_table(table: DatabaseTable) -> Self {
        let (table_name, columns) = table;
        let primary_keys = columns
            .iter()
            .filter(|(column_name, _, _)| column_name.ends_with("_id"))
            .map(|(column_name, _, _)| column_name.clone())
            .collect();

        Self {
            table_name,
            columns,
            primary_keys,
        }
    }
}

/// Define the Database struct
///
/// ## Fields
/// - pool: Arc<DatabasePool>
/// - entities: Arc<tokio::sync::Mutex<HashMap<String, EntityMetadata>>>
///
/// ## Methods
/// - new: Create a new Database
/// - close: Close the Database
#[derive(Debug)]
pub struct Database {
    pool: Arc<DatabasePool>,
    entities: Arc<tokio::sync::Mutex<HashMap<String, EntityMetadata>>>,
}

impl Database {
    /// Create a new Database
    ///
    /// ## Args
    /// - config: DbConfig
    ///
    /// ## Returns
    /// - Self if the Database is created successfully
    /// - Error otherwise
    ///
    /// ## Example
    /// ```rust,ignore
    /// extern crate cargoal;
    /// use cargoal::db::{Database, DbConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = DbConfig::from_env().unwrap();
    ///     let db = Database::new(config).await.unwrap();
    /// }
    pub async fn new(config: DbConfig) -> Result<Self, Error> {
        let pool = match config.db_type {
            DatabaseType::Postgres => {
                let pool = Pool::<Postgres>::connect(&config.database_url).await?;
                DatabasePool::Postgres(pool)
            }
            DatabaseType::MySql => {
                let pool = Pool::<MySql>::connect(&config.database_url).await?;
                DatabasePool::MySql(pool)
            }
            DatabaseType::Sqlite => {
                let pool = Pool::<Sqlite>::connect(&config.database_url).await?;
                DatabasePool::Sqlite(pool)
            }
        };

        DatabaseType::set_db_type(config.db_type); // Set the database type to be used by another orm functions

        Ok(Self {
            pool: Arc::new(pool),
            entities: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        })
    }

    /// Register an entity to the Database
    ///
    /// ## Args
    /// - T: EntityTrait
    pub async fn register_entity<T>(&self)
    where
        T: EntityTrait,
    {
        let table_name = T::TABLE_NAME;
        let columns = T::COLUMNS;
        let types = T::TYPES;
        let primary_keys = T::primary_keys();

        let mut column_definitions = Vec::new();
        for (column, ty) in columns.iter().zip(types.iter()) {
            column_definitions.push((column.to_string(), ty.to_string()));
        }

        let metadata = EntityMetadata {
            table_name: table_name.to_string(),
            columns: column_definitions
                .into_iter()
                .map(|(name, ty)| (name, ty, false))
                .collect(),
            primary_keys: primary_keys.into_iter().map(|s| s.to_string()).collect(),
        };

        let mut entities = self.entities.lock().await;
        entities.insert(metadata.table_name.clone(), metadata);
    }

    /// Close the Database
    pub async fn close(&self) {
        match &*self.pool {
            DatabasePool::Postgres(pool) => pool.close().await,
            DatabasePool::MySql(pool) => pool.close().await,
            DatabasePool::Sqlite(pool) => pool.close().await,
        }
    }

    /// Execute a query
    ///
    /// ## Args
    /// - query: &str
    ///
    /// ## Returns
    /// () if the query is executed successfully
    /// Error otherwise
    pub async fn execute(&self, query: &str) -> sqlx::Result<()> {
        match &*self.pool {
            DatabasePool::Postgres(pool) => sqlx::query(query).execute(pool).await.map(|_| ()),
            DatabasePool::MySql(pool) => sqlx::query(query).execute(pool).await.map(|_| ()),
            DatabasePool::Sqlite(pool) => sqlx::query(query).execute(pool).await.map(|_| ()),
        }
    }

    /// Fetch the tables metadata from the database (table name, columns name, data types, is nullable for each column)
    ///
    /// ## Returns
    /// - Vec<DatabaseTable>
    pub async fn fetch_tables_metadata(&self) -> Vec<DatabaseTable> {
        let query = r#"
            SELECT table_name, column_name, data_type, is_nullable
            FROM information_schema.columns
            WHERE table_schema = 'public'
            ORDER BY table_name, ordinal_position;
        "#;

        let rows = match &*self.pool {
            DatabasePool::Postgres(pool) => sqlx::query(query)
                .fetch_all(pool)
                .await
                .expect("Error retrieving metadata")
                .into_iter()
                .map(|row| {
                    (
                        row.get::<String, _>("table_name"),
                        row.get::<String, _>("column_name"),
                        row.get::<String, _>("data_type"),
                        row.get::<String, _>("is_nullable") == "YES",
                    )
                })
                .collect::<Vec<_>>(),
            DatabasePool::MySql(pool) => sqlx::query(query)
                .fetch_all(pool)
                .await
                .expect("Error retrieving metadata")
                .into_iter()
                .map(|row| {
                    (
                        row.get::<String, _>("table_name"),
                        row.get::<String, _>("column_name"),
                        row.get::<String, _>("data_type"),
                        row.get::<String, _>("is_nullable") == "YES",
                    )
                })
                .collect::<Vec<_>>(),
            DatabasePool::Sqlite(pool) => sqlx::query(query)
                .fetch_all(pool)
                .await
                .expect("Error retrieving metadata")
                .into_iter()
                .map(|row| {
                    (
                        row.get::<String, _>("table_name"),
                        row.get::<String, _>("column_name"),
                        row.get::<String, _>("data_type"),
                        row.get::<String, _>("is_nullable") == "YES",
                    )
                })
                .collect::<Vec<_>>(),
        };

        let mut tables: Vec<DatabaseTable> = Vec::new();

        for (table_name, column_name, data_type, is_nullable) in rows {
            if let Some(table) = tables.iter_mut().find(|t| t.0 == table_name) {
                table.1.push((column_name, data_type, is_nullable));
            } else {
                tables.push((table_name, vec![(column_name, data_type, is_nullable)]));
            }
        }

        tables
    }
}
