use cargoal::db::config::{DatabaseType, DbConfig};
use cargoal::db::connection::{Database, EntityTrait};
use cargoal_macros::Entity;

#[tokio::test]
async fn test_opening_connection() {
    // WARNING: This test will fail if the database is not running
    let db_config = DbConfig::new(
        DatabaseType::Postgres,
        "postgresql://admin:admin@localhost:5432/mydatabase".to_string(),
        None,
        None,
    );
    let db_result = Database::new(db_config).await;
    match db_result {
        Ok(db) => {
            db.close().await;
            assert!(true);
        }
        Err(_) => assert!(false),
    }
}

#[derive(Entity)]
pub struct User {
    #[column("id")]
    #[primary_key]
    pub id: i32,

    #[column("name")]
    pub name: String,

    #[column("email")]
    #[unique]
    pub email: String,
}

#[tokio::test]
async fn test_entity_macro() {
    assert_eq!(User::TABLE_NAME, "user");
    assert_eq!(User::COLUMNS, &["id", "name", "email"]);
    assert_eq!(User::TYPES, &["i32", "String", "String"]);
    assert_eq!(User::primary_keys(), vec!["id"]);
}

#[tokio::test]
async fn test_retrieve_db_type() {
    // WARNING: This test will fail if the database is not running
    let db_config = DbConfig::new(
        DatabaseType::Postgres,
        "postgresql://admin:admin@localhost:5432/mydatabase".to_string(),
        None,
        None,
    );
    let db_result = Database::new(db_config).await;
    match db_result {
        Ok(db) => {
            db.close().await;
            assert!(true);
        }
        Err(_) => assert!(false),
    }

    let given_type = DatabaseType::rust_type_to_sql_type("i32");
    assert_eq!(given_type, ("INTEGER".to_string(), false));
}

#[tokio::test]
async fn test_create_table() {
    // WARNING: This test will fail if the database is not running
    // WARNING: This test will create a table in the database. You should drop it after running this test
    let db_config = DbConfig::new(
        DatabaseType::Postgres,
        "postgresql://admin:admin@localhost:5432/mydatabase".to_string(),
        None,
        None,
    );
    let db_result = Database::new(db_config).await;
    match db_result {
        Ok(db) => {
            db.register_entity::<User>().await;
            let result = db.create_table("user").await;
            if let Err(e) = result {
                println!("Error: {}", e);
                assert!(false);
            }

            let metadata = db.fetch_tables_metadata().await;
            assert_eq!(metadata.len(), 1);
            assert!(true);
            db.close().await;
        }
        Err(_) => assert!(false),
    }
}
