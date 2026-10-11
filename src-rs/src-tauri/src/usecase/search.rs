use std::collections::{BTreeMap, HashMap};

use anydex_plugin_api_lib::vector_store::types::SearchRequest;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    dto::search::{
        DirectoryEntry, FileMetadata, GetFileInfoResult, GrepMatch, GrepResult, ListDirectoryResult,
    },
    plugin::{entry::VECTOR_STORE_MGR, vector_store::interface::VectorStore},
    repository::{json, sqlite},
};

const GREP_RESULT_LIMIT: i32 = 20;

pub async fn list_directory(
    index_id: Uuid,
    path: &str,
    database: &DatabaseConnection,
) -> Result<ListDirectoryResult, String> {
    let path = normalize_path(path)?;
    let files = get_indexed_files(index_id, database).await?;
    let mut entries = BTreeMap::<String, bool>::new();
    let prefix = if path == "/" {
        "/".to_owned()
    } else {
        format!("{path}/")
    };

    for file in files {
        let file_path = normalize_path(&file.file_path)?;
        let Some(relative_path) = file_path.strip_prefix(&prefix) else {
            continue;
        };
        if relative_path.is_empty() {
            continue;
        }

        if let Some((directory, _)) = relative_path.split_once('/') {
            let directory_path = join_path(&path, directory);
            entries.insert(directory_path, true);
        } else {
            entries.insert(file_path, false);
        }
    }

    Ok(ListDirectoryResult {
        entries: entries
            .into_iter()
            .map(|(path, is_directory)| DirectoryEntry {
                name: path.rsplit('/').next().unwrap_or_default().to_owned(),
                path,
                is_directory,
            })
            .collect(),
    })
}

pub async fn grep(
    index_id: Uuid,
    pattern: &str,
    target: &str,
    database: &DatabaseConnection,
) -> Result<GrepResult, String> {
    if pattern.trim().is_empty() {
        return Err("search pattern must not be empty".to_owned());
    }
    let target = normalize_path(target)?;
    let files = get_indexed_files(index_id, database).await?;
    let mut target_files = Vec::new();
    for file in files {
        let file_path = normalize_path(&file.file_path)?;
        if file_path == target || target == "/" || file_path.starts_with(&format!("{target}/")) {
            target_files.push(file);
        }
    }
    if target_files.is_empty() {
        return Ok(GrepResult {
            matches: Vec::new(),
        });
    }

    let userdata = json::load()?;
    let index = userdata
        .indexes
        .iter()
        .find(|index| index.id == index_id)
        .ok_or_else(|| format!("index {index_id} was not found"))?;
    let vector_store_config = index
        .vector_store
        .as_ref()
        .ok_or_else(|| format!("index {index_id} has no vector store configured"))?;

    let vector_store = {
        let mut manager = VECTOR_STORE_MGR.lock().map_err(|err| err.to_string())?;
        manager
            .as_mut()
            .ok_or_else(|| "vector store plugin manager is not initialized".to_owned())?
            .get_plugin(&vector_store_config.name)
            .ok_or_else(|| {
                format!(
                    "vector store plugin {} was not found in the plugin manager",
                    vector_store_config.name
                )
            })?
            .clone()
    };

    let vector_store_params = vector_store_config.params.clone();
    let search_text = pattern.to_owned();
    let file_ids = target_files.iter().map(|file| file.id).collect();
    let hits = tokio::task::spawn_blocking(move || {
        let vector_store_handler =
            vector_store.get_handler(&vector_store_params, index_id.to_string())?;
        vector_store.search(
            vector_store_handler.handler,
            &SearchRequest {
                text: Some(search_text),
                embedding: None,
                target_file_ids: file_ids,
                offset: 0,
                limit: GREP_RESULT_LIMIT,
            },
        )
    })
    .await
    .map_err(|err| format!("file search task failed: {err}"))??;

    let files_by_id: HashMap<_, _> = target_files
        .into_iter()
        .map(|file| (file.id, file))
        .collect();
    let matches = hits
        .into_iter()
        .map(|hit| {
            let file = files_by_id.get(&hit.chunk.file_id).ok_or_else(|| {
                format!(
                    "vector store returned an unknown file id: {}",
                    hit.chunk.file_id
                )
            })?;
            Ok(GrepMatch {
                file: to_file_metadata(file),
                content: hit.chunk.content,
                score: hit.score,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(GrepResult { matches })
}

pub async fn get_file_info(
    index_id: Uuid,
    uri: &str,
    database: &DatabaseConnection,
) -> Result<GetFileInfoResult, String> {
    let file = sqlite::models::file::Entity::find()
        .filter(sqlite::models::file::Column::IndexId.eq(index_id.to_string()))
        .filter(sqlite::models::file::Column::FileUri.eq(uri))
        .one(database)
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("file with URI '{uri}' was not found in index {index_id}"))?;

    Ok(GetFileInfoResult {
        file: to_file_metadata(&file),
    })
}

async fn get_indexed_files(
    index_id: Uuid,
    database: &DatabaseConnection,
) -> Result<Vec<sqlite::models::file::Model>, String> {
    sqlite::models::file::Entity::find()
        .filter(sqlite::models::file::Column::IndexId.eq(index_id.to_string()))
        .all(database)
        .await
        .map_err(|err| err.to_string())
}

fn to_file_metadata(file: &sqlite::models::file::Model) -> FileMetadata {
    FileMetadata {
        id: file.id.to_string(),
        index_id: file.index_id.clone(),
        name: file.name.clone(),
        file_uri: file.file_uri.clone(),
        file_path: file.file_path.clone(),
        file_dir: file.file_dir.clone(),
        file_name: file.file_name.clone(),
        content_type: file.content_type.clone(),
        size: file.size.to_string(),
        updated_at: file.updated_at.clone(),
        file_hash: file.file_hash.clone(),
    }
}

fn normalize_path(path: &str) -> Result<String, String> {
    let path = path.replace('\\', "/");
    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => return Err("path must not contain '..' components".to_owned()),
            component => components.push(component),
        }
    }
    if components.is_empty() {
        Ok("/".to_owned())
    } else {
        Ok(format!("/{}", components.join("/")))
    }
}

fn join_path(directory: &str, name: &str) -> String {
    if directory == "/" {
        format!("/{name}")
    } else {
        format!("{directory}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::{join_path, normalize_path};

    #[test]
    fn normalize_path_canonicalizes_separators_and_rejects_parent_components() {
        assert_eq!(normalize_path("docs\\notes").unwrap(), "/docs/notes");
        assert_eq!(normalize_path("/").unwrap(), "/");
        assert!(normalize_path("../secret").is_err());
    }

    #[test]
    fn join_path_handles_root() {
        assert_eq!(join_path("/", "notes"), "/notes");
        assert_eq!(join_path("/docs", "notes"), "/docs/notes");
    }
}
