use std::{
    fs,
    path::{Path, PathBuf},
};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    path::get_dir,
    plugin::{entry::FILE_SOURCE_MGR, file_source::interface::FileSource},
    repository::{json::load, sqlite},
    usecase::oauth::get_authentication,
};

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

pub async fn download_file(
    index_id: Uuid,
    file_uri: &str,
    database: &DatabaseConnection,
) -> Result<PathBuf, String> {
    let userdata = load()?;
    let index = userdata
        .indexes
        .iter()
        .find(|v| v.id == index_id)
        .ok_or("index not found")?;
    let file_source = index.file_source.clone().ok_or("file source not found")?;
    let file_source_name = &file_source.name;
    let plugin = {
        let mut manager = FILE_SOURCE_MGR.lock().unwrap();
        let manager = manager
            .as_mut()
            .ok_or("file source plugin manager is not initialized".to_owned())?;
        manager
            .get_plugin(file_source_name)
            .ok_or(format!(
                "File Source plugin {file_source_name} was not found in the plugin manager"
            ))?
            .clone()
    };
    let handler = plugin.get_handler(&file_source.params)?;
    let cache_dir = get_dir().app_cache_dir.join(index_id.to_string());
    let access_token = match file_source.oauth_id {
        Some(oauth_client_id) => {
            let mut mdl = get_authentication(index_id, oauth_client_id, database).await?;
            match mdl.as_mut() {
                Some(auth) => Some(auth.load_access_token().await?),
                None => None,
            }
        }
        None => None,
    };
    fs::create_dir_all(&cache_dir).map_err(|err| err.to_string())?;
    let downloaded_path = plugin.download(
        handler.handler,
        file_uri,
        access_token,
        &cache_dir.to_str().unwrap(),
    )?;
    Ok(Path::new(&downloaded_path).to_path_buf())
}
