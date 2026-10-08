use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::repository::sqlite;

pub async fn get_all_indexed_files(
    index_id: Uuid,
    database: &DatabaseConnection,
) -> Result<Vec<sqlite::models::file::Model>, String> {
    let files = sqlite::models::file::Entity::find()
        .filter(sqlite::models::file::Column::IndexId.eq(index_id.to_string()))
        .all(database)
        .await
        .map_err(|err| err.to_string())?;
    Ok(files)
}

pub async fn delete_files(
    index_id: Uuid,
    file_uris: &[String],
    database: &DatabaseConnection,
) -> Result<(), String> {
    sqlite::models::file::Entity::delete_many()
        .filter(sqlite::models::file::Column::IndexId.eq(index_id.to_string()))
        .filter(sqlite::models::file::Column::FileUri.is_in(file_uris))
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}
