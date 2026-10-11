use std::path::Path;

use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Schema,
    TransactionTrait,
};
use url::Url;

use crate::path::get_dir;
pub mod authentication;
pub mod job;
pub mod models;

pub fn database_url(database_path: &Path) -> Result<String, DbErr> {
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

pub async fn init() -> Result<DatabaseConnection, String> {
    let database_path = get_dir().app_data_dir.join("anydex.sqlite3");
    let database_url = database_url(&database_path).map_err(|err| err.to_string())?;
    let mut options = ConnectOptions::new(&database_url);
    // This is a local embedded database. A single writer avoids SQLite lock
    // contention while commands access the connection managed by Tauri.
    options.max_connections(1);
    options.sqlx_logging(false);

    let database = Database::connect(options)
        .await
        .map_err(|err| err.to_string())?;
    let transaction = database.begin().await.map_err(|err| err.to_string())?;
    let schema = Schema::new(DbBackend::Sqlite);

    macro_rules! create_entity_schema {
        ($entity:expr) => {{
            let mut table = schema.create_table_from_entity($entity);
            table.if_not_exists();
            transaction
                .execute(&table)
                .await
                .map_err(|err| err.to_string())?;

            for mut index in schema.create_index_from_entity($entity) {
                index.if_not_exists();
                transaction
                    .execute(&index)
                    .await
                    .map_err(|err| err.to_string())?;
            }
        }};
    }

    // Create referenced tables before their dependants so SQLite can enforce
    // the foreign keys generated from the entity relations.
    create_entity_schema!(models::oauth_authentication::Entity);
    create_entity_schema!(models::file::Entity);
    create_entity_schema!(models::job::Entity);
    create_entity_schema!(models::indexing_history::Entity);

    transaction.commit().await.map_err(|err| err.to_string())?;
    Ok(database)
}
