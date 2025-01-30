use sqlx::{Error, MySql, Pool, Postgres, Sqlite};
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
}
