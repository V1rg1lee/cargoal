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
    /// Create a new EntityMetadata
    ///
    /// ## Args
    /// - table_name: String
    /// - columns: ColumnInfos
    /// - primary_keys: Vec<String>
    ///
    /// ## Returns
    /// - Self
    pub fn new(table_name: String, columns: ColumnInfos, primary_keys: Vec<String>) -> Self {
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

    /// Fetch the tables metadata from the database (table name, columns name, data types, is nullable, is a pk for each column)
    ///
    /// ## Returns
    /// - Vec<EntityMetadata>
    pub async fn fetch_tables_metadata(&self) -> Vec<EntityMetadata> {
        let query = r#"
            SELECT c.table_name, 
                c.column_name, 
                c.data_type, 
                c.is_nullable, 
                CASE 
                    WHEN kcu.column_name IS NOT NULL THEN true 
                    ELSE false 
                END AS is_primary_key
            FROM information_schema.columns c
            LEFT JOIN information_schema.key_column_usage kcu 
                ON c.table_name = kcu.table_name 
                AND c.column_name = kcu.column_name
                AND c.table_schema = kcu.table_schema
            WHERE c.table_schema = 'public'
            ORDER BY c.table_name, c.ordinal_position;
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
                        row.get::<bool, _>("is_primary_key"),
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
                        row.get::<bool, _>("is_primary_key"),
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
                        row.get::<bool, _>("is_primary_key"),
                    )
                })
                .collect::<Vec<_>>(),
        };

        let mut tables: HashMap<String, ColumnInfos> = HashMap::new();

        let mut primary_keys = Vec::<String>::new();

        for row in rows {
            let table_name: String = row.0;
            let column_name: String = row.1;
            let data_type: String = row.2;
            let is_nullable: bool = row.3;
            let is_primary_key: bool = row.4;

            if is_primary_key {
                primary_keys.push(column_name.clone());
            }

            tables.entry(table_name.clone()).or_default().push((
                column_name,
                data_type,
                is_nullable,
            ));
        }

        tables
            .into_iter()
            .map(|(table_name, columns)| {
                EntityMetadata::new(table_name, columns, primary_keys.clone())
            })
            .collect()
    }

    /// Get the metadata of an entity
    /// 
    /// ## Args
    /// - table_name: &str
    /// 
    /// ## Returns
    /// - Option<EntityMetadata>
    async fn get_entity_metadata(&self, table_name: &str) -> Option<EntityMetadata> {
        let entities = self.entities.lock().await;
        entities.get(table_name).cloned()
    }

    /// Create a table in the database
    /// 
    /// ## Args
    /// - table_name: &str
    /// 
    /// ## Returns
    /// - () if the table is created successfully
    /// - Error otherwise
    pub async fn create_table(&self, table_name: &str) -> Result<(), Error> {
        if let Some(metadata) = self.get_entity_metadata(table_name).await {
            let columns = metadata.columns;

            let mut query = format!("CREATE TABLE IF NOT EXISTS \"{}\" (", table_name);
            for (i, (column_name, data_type, is_nullable)) in columns.iter().enumerate() {
                let db_type = DatabaseType::rust_type_to_sql_type(data_type);
                query.push_str(&format!(
                    "{} {} {}",
                    column_name,
                    db_type.0,
                    if *is_nullable { "NULL" } else { "NOT NULL" }
                ));

                if i < columns.len() - 1 {
                    query.push_str(", ");
                }
            }
            let primary_keys = metadata.primary_keys;
            if !primary_keys.is_empty() {
                query.push_str(", PRIMARY KEY (");
                for (i, pk) in primary_keys.iter().enumerate() {
                    query.push_str(pk);
                    if i < primary_keys.len() - 1 {
                        query.push_str(", ");
                    }
                }
                query.push(')');
            }
            query.push_str(");");

            println!("{}", query);

            self.execute(&query).await
        } else {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Table not found",
            )))
        }
    }
}
