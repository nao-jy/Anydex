//! Database connection and schema initialization.

use std::path::Path;

use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Schema,
    TransactionTrait,
};
use url::Url;

pub mod indexes;
pub mod models;

/// Builds a SQLite URL for a database file, escaping paths such as those that
/// contain spaces. `mode=rwc` lets SQLite create the file on first launch.
pub fn sqlite_database_url(database_path: &Path) -> Result<String, DbErr> {
    let file_url = Url::from_file_path(database_path).map_err(|_| {
        DbErr::Custom(format!(
            "database path cannot be represented as a file URL: {}",
            database_path.display()
        ))
    })?;
    let path = file_url
        .as_str()
        .strip_prefix("file:")
        .expect("Url::from_file_path must produce a file URL");

    Ok(format!("sqlite:{path}?mode=rwc"))
}

/// Opens the application database and creates the current schema when needed.
///
/// This operation is idempotent, so it is safe to run every time the app
/// starts. SQLite's SQLx driver enables foreign-key enforcement for each
/// connection by default.
pub async fn initialize(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(database_url);
    // This is a local embedded database. A single writer avoids SQLite lock
    // contention while commands access the connection managed by Tauri.
    options.max_connections(1);
    options.sqlx_logging(false);

    let database = Database::connect(options).await?;
    let transaction = database.begin().await?;
    let schema = Schema::new(DbBackend::Sqlite);

    macro_rules! create_entity_schema {
        ($entity:expr) => {{
            let mut table = schema.create_table_from_entity($entity);
            table.if_not_exists();
            transaction.execute(&table).await?;

            for mut index in schema.create_index_from_entity($entity) {
                index.if_not_exists();
                transaction.execute(&index).await?;
            }
        }};
    }

    // Create referenced tables before their dependants so SQLite can enforce
    // the foreign keys generated from the entity relations.
    create_entity_schema!(models::index::Entity);
    create_entity_schema!(models::embedder::Entity);
    create_entity_schema!(models::vector_store::Entity);
    create_entity_schema!(models::indexer::Entity);
    create_entity_schema!(models::file_source::Entity);
    create_entity_schema!(models::schedule::Entity);
    create_entity_schema!(models::oauth_client::Entity);
    create_entity_schema!(models::oauth_authentication::Entity);
    create_entity_schema!(models::file::Entity);
    create_entity_schema!(models::task::Entity);

    transaction.commit().await?;
    Ok(database)
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    use super::initialize;

    #[tokio::test]
    async fn initializes_all_application_tables() {
        let database = initialize("sqlite::memory:").await.unwrap();

        for table in [
            "indexes",
            "embedders",
            "vector_stores",
            "indexers",
            "file_sources",
            "schedules",
            "oauth_clients",
            "oauth_authentications",
            "files",
            "tasks",
        ] {
            let statement = Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?",
                [table.into()],
            );
            assert!(database.query_one_raw(statement).await.unwrap().is_some());
        }
    }
}
