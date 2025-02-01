use sqlx::{Error, MySql, Pool, Postgres, Row, Sqlite};
use std::sync::Arc;

use super::config::{DatabaseType, DbConfig};

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

/// Define the Database struct
///
/// ## Fields
/// - pool: Arc<DatabasePool>
///
/// ## Methods
/// - new: Create a new Database
/// - close: Close the Database
#[derive(Debug)]
pub struct Database {
    pool: Arc<DatabasePool>,
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
        })
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
    pub async fn execute(&self, query: &str) -> Result<sqlx::Result<()>, Error> {
        let result = match &*self.pool {
            DatabasePool::Postgres(pool) => sqlx::query(query).execute(pool).await.map(|_| ()),
            DatabasePool::MySql(pool) => sqlx::query(query).execute(pool).await.map(|_| ()),
            DatabasePool::Sqlite(pool) => sqlx::query(query).execute(pool).await.map(|_| ()),
        };
        Ok(result.map_err(|e| e.into()))
    }

    /// Fetch the tables metadata from the database (table name, columns name, data types, is nullable for each column)
    ///
    /// ## Returns
    /// - Vec<(String, Vec<(String, String, bool)>)>
    pub async fn fetch_tables_metadata(&self) -> Vec<(String, Vec<(String, String, bool)>)> {
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

        let mut tables: Vec<(String, Vec<(String, String, bool)>)> = Vec::new();

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
