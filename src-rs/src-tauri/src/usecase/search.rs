use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, OpenOptions},
    io::Write,
};

use anydex_plugin_api_lib::{
    embedder::interface::DynamicEmbedder, vector_store::types::SearchRequest,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    dto::search::{
        DirectoryEntry, FileMetadata, GetFileInfoResult, GrepResult, ListDirectoryResult,
        SearchMatch, SemanticSearchQuery, SemanticSearchResult,
    },
    path,
    plugin::{
        embedder::interface::Embedder,
        entry::{EMBEDDER_MGR, VECTOR_STORE_MGR},
        vector_store::interface::VectorStore,
    },
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
    let mut entries = BTreeMap::<String, DirectoryEntry>::new();
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
            match entries.entry(directory_path.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(DirectoryEntry {
                        name: directory.to_owned(),
                        path: directory_path,
                        is_directory: true,
                        file_uri: None,
                        size: file.size.to_string(),
                        updated_at: file.updated_at.clone(),
                    });
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    let directory_entry = entry.get_mut();
                    let size = directory_entry
                        .size
                        .parse::<i64>()
                        .map_err(|err| err.to_string())?;
                    directory_entry.size = size
                        .checked_add(file.size)
                        .ok_or_else(|| "directory size exceeds the supported range".to_owned())?
                        .to_string();
                    if file.updated_at.as_ref().is_some_and(|updated_at| {
                        directory_entry
                            .updated_at
                            .as_ref()
                            .is_none_or(|current| updated_at > current)
                    }) {
                        directory_entry.updated_at = file.updated_at.clone();
                    }
                }
            }
        } else {
            entries.insert(
                file_path.clone(),
                DirectoryEntry {
                    name: file_path.rsplit('/').next().unwrap_or_default().to_owned(),
                    path: file_path,
                    is_directory: false,
                    file_uri: Some(file.file_uri),
                    size: file.size.to_string(),
                    updated_at: file.updated_at,
                },
            );
        }
    }

    Ok(ListDirectoryResult {
        entries: entries.into_values().collect(),
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
    let target_files = get_target_files(index_id, &target, database).await?;
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
    let file_ids: Vec<i64> = target_files.iter().map(|file| file.id).collect();
    let hits = tokio::task::spawn_blocking(move || {
        let vector_store_handler = vector_store.get_handler(
            &vector_store_params,
            index_id.to_string(),
            &path::get_dir().app_data_dir.to_string_lossy().to_string(),
        )?;
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
            let file_id = i64::try_from(hit.chunk.file_id).map_err(|err| err.to_string())?;
            let file = files_by_id.get(&file_id).ok_or_else(|| {
                format!(
                    "vector store returned an unknown file id: {}",
                    hit.chunk.file_id
                )
            })?;
            Ok(SearchMatch {
                file: to_file_metadata(file),
                content: hit.chunk.content,
                score: hit.score,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(GrepResult { matches })
}

pub async fn semantic_search(
    index_id: Uuid,
    query: SemanticSearchQuery,
    target: &str,
    database: &DatabaseConnection,
) -> Result<SemanticSearchResult, String> {
    if let SemanticSearchQuery::Text(text) = &query {
        if text.trim().is_empty() {
            return Err("search query must not be empty".to_owned());
        }
    }
    let target = normalize_path(target)?;
    let target_files = get_target_files(index_id, &target, database).await?;
    if target_files.is_empty() {
        return Ok(SemanticSearchResult {
            matches: Vec::new(),
        });
    }

    let userdata = json::load()?;
    let index = userdata
        .indexes
        .iter()
        .find(|index| index.id == index_id)
        .ok_or_else(|| format!("index {index_id} was not found"))?;
    let embedder_config = index
        .embedder
        .as_ref()
        .ok_or_else(|| format!("index {index_id} has no embedder configured"))?;
    let vector_store_config = index
        .vector_store
        .as_ref()
        .ok_or_else(|| format!("index {index_id} has no vector store configured"))?;

    let embedder = {
        let mut manager = EMBEDDER_MGR.lock().map_err(|err| err.to_string())?;
        manager
            .as_mut()
            .ok_or_else(|| "embedder plugin manager is not initialized".to_owned())?
            .get_plugin(&embedder_config.name)
            .ok_or_else(|| {
                format!(
                    "embedder plugin {} was not found in the plugin manager",
                    embedder_config.name
                )
            })?
            .clone()
    };
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

    let embedder_params = embedder_config.params.clone();
    let vector_store_params = vector_store_config.params.clone();
    let file_ids: Vec<i64> = target_files.iter().map(|file| file.id).collect();
    let hits = tokio::task::spawn_blocking(move || {
        let embedding = embed_query(&embedder, &embedder_params, query)?;
        let vector_store_handler = vector_store.get_handler(
            &vector_store_params,
            index_id.to_string(),
            &path::get_dir().app_data_dir.to_string_lossy().to_string(),
        )?;
        vector_store.search(
            vector_store_handler.handler,
            &SearchRequest {
                text: None,
                embedding: Some(embedding),
                target_file_ids: file_ids,
                offset: 0,
                limit: GREP_RESULT_LIMIT,
            },
        )
    })
    .await
    .map_err(|err| format!("semantic search task failed: {err}"))??;

    let files_by_id: HashMap<_, _> = target_files
        .into_iter()
        .map(|file| (file.id, file))
        .collect();
    let matches = hits
        .into_iter()
        .map(|hit| {
            let file_id = i64::try_from(hit.chunk.file_id).map_err(|err| err.to_string())?;
            let file = files_by_id.get(&file_id).ok_or_else(|| {
                format!(
                    "vector store returned an unknown file id: {}",
                    hit.chunk.file_id
                )
            })?;
            Ok(SearchMatch {
                file: to_file_metadata(file),
                content: hit.chunk.content,
                score: hit.score,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(SemanticSearchResult { matches })
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

async fn get_target_files(
    index_id: Uuid,
    target: &str,
    database: &DatabaseConnection,
) -> Result<Vec<sqlite::models::file::Model>, String> {
    get_indexed_files(index_id, database)
        .await?
        .into_iter()
        .filter_map(|file| match normalize_path(&file.file_path) {
            Ok(file_path)
                if file_path == target
                    || target == "/"
                    || file_path.starts_with(&format!("{target}/")) =>
            {
                Some(Ok(file))
            }
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        })
        .collect()
}

fn to_file_metadata(file: &sqlite::models::file::Model) -> FileMetadata {
    FileMetadata {
        index_id: file.index_id.clone(),
        name: file.name.clone(),
        file_uri: file.file_uri.clone(),
        content_type: file.content_type.clone(),
        size: file.size.to_string(),
        updated_at: file.updated_at.clone(),
        file_hash: file.file_hash.clone(),
    }
}

fn embed_query(
    embedder: &DynamicEmbedder,
    params: &serde_json::Value,
    query: SemanticSearchQuery,
) -> Result<Vec<f32>, String> {
    let (extension, contents) = query_contents(query)?;

    let path = std::env::temp_dir().join(format!("anydex-search-{}.{}", Uuid::now_v7(), extension));
    let mut created = false;
    let embedding_result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|err| format!("failed to create temporary search query: {err}"))?;
        created = true;
        file.write_all(&contents)
            .map_err(|err| format!("failed to write temporary search query: {err}"))?;
        let path = path
            .to_str()
            .ok_or_else(|| "temporary search query path is not valid UTF-8".to_owned())?;
        let handler = embedder.get_handler(params)?;
        embedder.embed(handler.handler, path)
    })();

    if created {
        match fs::remove_file(&path) {
            Ok(()) => embedding_result,
            Err(cleanup_err) => match embedding_result {
                Ok(_) => Err(format!(
                    "failed to remove temporary search query: {cleanup_err}"
                )),
                Err(err) => {
                    tracing::warn!("failed to remove temporary search query: {cleanup_err}");
                    Err(err)
                }
            },
        }
    } else {
        embedding_result
    }
}

fn query_contents(query: SemanticSearchQuery) -> Result<(&'static str, Vec<u8>), String> {
    match query {
        SemanticSearchQuery::Text(text) => Ok(("txt", text.into_bytes())),
        SemanticSearchQuery::ImageBase64(encoded) => {
            let encoded = encoded
                .split_once(',')
                .filter(|(header, _)| {
                    header.starts_with("data:image/") && header.ends_with(";base64")
                })
                .map_or(encoded.as_str(), |(_, data)| data);
            let contents = STANDARD
                .decode(encoded)
                .map_err(|err| format!("invalid base64 image query: {err}"))?;
            let extension = image_extension(&contents)
                .ok_or_else(|| "unsupported or invalid base64 image query".to_owned())?;
            Ok((extension, contents))
        }
    }
}

fn image_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("jpg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("gif")
    } else if bytes.starts_with(b"BM") {
        Some("bmp")
    } else if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        Some("tiff")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
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
    use super::{image_extension, join_path, normalize_path, query_contents};
    use crate::dto::search::SemanticSearchQuery;

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

    #[test]
    fn query_contents_supports_text_and_base64_images() {
        assert_eq!(
            query_contents(SemanticSearchQuery::Text("hello".to_owned())).unwrap(),
            ("txt", b"hello".to_vec())
        );

        let (extension, bytes) =
            query_contents(SemanticSearchQuery::ImageBase64("iVBORw0KGgo=".to_owned())).unwrap();
        assert_eq!(extension, "png");
        assert_eq!(image_extension(&bytes), Some("png"));

        let (extension, bytes) = query_contents(SemanticSearchQuery::ImageBase64(
            "data:image/png;base64,iVBORw0KGgo=".to_owned(),
        ))
        .unwrap();
        assert_eq!(extension, "png");
        assert_eq!(image_extension(&bytes), Some("png"));
    }

    #[test]
    fn query_contents_rejects_invalid_image_base64() {
        assert!(
            query_contents(SemanticSearchQuery::ImageBase64("not an image".to_owned())).is_err()
        );
    }
}
