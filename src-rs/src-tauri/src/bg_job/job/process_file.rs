use anydex_plugin_api_lib::file_source::types::OutputFileInfo;
use anydex_plugin_api_lib::vector_store::types::{Chunk, FileId, IndexId};
use mime_guess::mime;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path};
use uuid::Uuid;

use crate::bg_job::entry::get_tmp_dir;
use crate::{
    bg_job::job::Job,
    plugin::{
        embedder::interface::Embedder,
        entry::{EMBEDDER_MGR, FILE_EXTRACTOR_MGR, FILE_SOURCE_MGR, VECTOR_STORE_MGR},
        file_extractor::interface::FileExtractor,
        file_source::interface::FileSource,
        vector_store::interface::VectorStore,
    },
    repository::{self, sqlite},
    usecase::oauth::get_authentication,
};

pub struct ProcessFile {
    pub db: DatabaseConnection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessFileArgs {
    pub index_id: Uuid,
    pub file_info: OutputFileInfo,
    pub file_source: repository::json::models::FileSource,
    pub file_extractor: repository::json::models::FileExtractor,
    pub embedder: repository::json::models::Embedder,
    pub vector_store: repository::json::models::VectorStore,
}

impl Job for ProcessFile {
    type JobArgsType = ProcessFileArgs;
    async fn process(&self, payload: Value) -> Result<(), String> {
        let args: ProcessFileArgs =
            serde_json::from_value(payload).map_err(|err| err.to_string())?;

        let userdata = repository::json::load()?;
        let plugin_index_id = to_plugin_index_id(args.index_id);
        if userdata.indexes.iter().any(|index| {
            index.id != args.index_id && to_plugin_index_id(index.id) == plugin_index_id
        }) {
            return Err(format!(
                "index ID collision in vector store for index {}",
                args.index_id
            ));
        }

        let existing_file = sqlite::models::file::Entity::find()
            .filter(sqlite::models::file::Column::IndexId.eq(args.index_id.to_string()))
            .filter(sqlite::models::file::Column::FileUri.eq(args.file_info.file_uri.clone()))
            .one(&self.db)
            .await
            .map_err(|err| err.to_string())?;
        let is_new_file = existing_file.is_none();
        let file = match existing_file {
            Some(file) => file,
            None => new_file_model(args.index_id, &args.file_info)?
                .insert(&self.db)
                .await
                .map_err(|err| err.to_string())?,
        };
        let file_id = match i32::try_from(file.id) {
            Ok(file_id) => file_id,
            Err(err) => {
                if is_new_file {
                    if let Err(delete_err) = sqlite::models::file::Entity::delete_by_id(file.id)
                        .exec(&self.db)
                        .await
                    {
                        return Err(format!(
                            "file ID {} cannot be used by the vector store: {err}; failed to remove the new file record: {delete_err}",
                            file.id
                        ));
                    }
                }
                return Err(format!(
                    "file ID {} cannot be used by the vector store: {err}",
                    file.id
                ));
            }
        };

        let process_result = process_file_plugins(&args, plugin_index_id, file_id, &self.db).await;
        if let Err(err) = process_result {
            if is_new_file {
                if let Err(delete_err) = sqlite::models::file::Entity::delete_by_id(file.id)
                    .exec(&self.db)
                    .await
                {
                    return Err(format!(
                        "{err}; failed to remove the new file record: {delete_err}"
                    ));
                }
            }
            return Err(err);
        }

        let mut active_file: sqlite::models::file::ActiveModel = file.into();
        set_file_info(&mut active_file, args.index_id, &args.file_info)?;
        active_file
            .update(&self.db)
            .await
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

async fn process_file_plugins(
    args: &ProcessFileArgs,
    index_id: IndexId,
    file_id: FileId,
    db: &DatabaseConnection,
) -> Result<(), String> {
    let oauth_token = match args.file_source.oauth_id {
        Some(oauth_id) => {
            let mut authentication = get_authentication(args.index_id, oauth_id, db)
                .await?
                .ok_or_else(|| {
                    "OAuth is configured but the index is not authenticated".to_owned()
                })?;
            Some(authentication.load_access_token().await?)
        }
        None => None,
    };

    let base_dir = get_tmp_dir()?.join("files").join(args.index_id.to_string());
    let downloaded_dir = base_dir.join("downloads");
    let extracted_dir = base_dir.join("extracted");
    std::fs::create_dir_all(&downloaded_dir).map_err(|err| err.to_string())?;
    std::fs::create_dir_all(&extracted_dir).map_err(|err| err.to_string())?;
    let downloaded_dir = downloaded_dir
        .to_str()
        .ok_or_else(|| "download directory path is not valid UTF-8".to_owned())?
        .to_owned();
    let extracted_dir_string = extracted_dir
        .to_str()
        .ok_or_else(|| "extraction directory path is not valid UTF-8".to_owned())?
        .to_owned();

    let args = args.clone();
    tokio::task::spawn_blocking(move || {
        process_file_plugins_sync(
            &args,
            index_id,
            file_id,
            oauth_token,
            &downloaded_dir,
            &extracted_dir_string,
        )
    })
    .await
    .map_err(|err| err.to_string())?
}

fn process_file_plugins_sync(
    args: &ProcessFileArgs,
    index_id: IndexId,
    file_id: FileId,
    oauth_token: Option<String>,
    downloaded_dir: &str,
    extracted_dir_string: &str,
) -> Result<(), String> {
    let extracted_dir = Path::new(extracted_dir_string);
    let file_source = {
        let mut manager = FILE_SOURCE_MGR.lock().map_err(|err| err.to_string())?;
        manager
            .as_mut()
            .ok_or_else(|| "file source plugin manager is not initialized".to_owned())?
            .get_plugin(&args.file_source.name)
            .ok_or_else(|| {
                format!(
                    "file source plugin {} was not found in the plugin manager",
                    args.file_source.name
                )
            })?
            .clone()
    };
    let file_extractor = {
        let mut manager = FILE_EXTRACTOR_MGR.lock().map_err(|err| err.to_string())?;
        manager
            .as_mut()
            .ok_or_else(|| "file extractor plugin manager is not initialized".to_owned())?
            .get_plugin(&args.file_extractor.name)
            .ok_or_else(|| {
                format!(
                    "file extractor plugin {} was not found in the plugin manager",
                    args.file_extractor.name
                )
            })?
            .clone()
    };
    let embedder = {
        let mut manager = EMBEDDER_MGR.lock().map_err(|err| err.to_string())?;
        manager
            .as_mut()
            .ok_or_else(|| "embedder plugin manager is not initialized".to_owned())?
            .get_plugin(&args.embedder.name)
            .ok_or_else(|| {
                format!(
                    "embedder plugin {} was not found in the plugin manager",
                    args.embedder.name
                )
            })?
            .clone()
    };
    let vector_store = {
        let mut manager = VECTOR_STORE_MGR.lock().map_err(|err| err.to_string())?;
        manager
            .as_mut()
            .ok_or_else(|| "vector store plugin manager is not initialized".to_owned())?
            .get_plugin(&args.vector_store.name)
            .ok_or_else(|| {
                format!(
                    "vector store plugin {} was not found in the plugin manager",
                    args.vector_store.name
                )
            })?
            .clone()
    };

    let file_source_handler = file_source.get_handler(&args.file_source.params)?;
    let file_extractor_handler =
        file_extractor.get_handler(&args.file_extractor.params, &extracted_dir_string)?;
    let embedder_handler = embedder.get_handler(&args.embedder.params)?;
    let vector_store_handler = vector_store.get_handler(&args.vector_store.params, index_id)?;

    let downloaded_file = file_source.download(
        file_source_handler.handler,
        &args.file_info.file_uri,
        oauth_token,
        downloaded_dir,
    )?;
    if !Path::new(&downloaded_file).is_file() {
        return Err(format!(
            "file source did not create a downloaded file at {downloaded_file}"
        ));
    }

    let extracted_files =
        file_extractor.extract(file_extractor_handler.handler, &downloaded_file)?;
    let mut chunks = Vec::with_capacity(extracted_files.len());
    for (chunk_index, relative_path) in extracted_files.iter().enumerate() {
        let relative_path = Path::new(relative_path);
        if relative_path.is_absolute()
            || relative_path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(format!(
                "file extractor returned an invalid relative path: {}",
                relative_path.display()
            ));
        }
        let extracted_file = extracted_dir.join(relative_path);
        if !extracted_file.is_file() {
            return Err(format!(
                "file extractor output does not exist: {}",
                extracted_file.display()
            ));
        }

        let file_mime = mime_guess::from_path(&extracted_file).first_or_octet_stream();
        let is_image_file = file_mime.type_() == mime::IMAGE;
        let content = if is_image_file {
            format!("[Image: {}]", relative_path.display())
        } else {
            std::fs::read_to_string(&extracted_file).map_err(|err| err.to_string())?
        };
        let content_vector = embedder.embed(
            embedder_handler.handler,
            extracted_file
                .to_str()
                .ok_or_else(|| "extracted file path is not valid UTF-8".to_owned())?,
        )?;
        chunks.push(Chunk {
            id: i32::try_from(chunk_index)
                .map_err(|err| format!("too many extracted chunks: {err}"))?,
            file_id,
            indexer_id: 0,
            content,
            content_vector,
        });
        fs::remove_file(extracted_file).map_err(|err| err.to_string())?;
    }

    vector_store.delete_chunks_file(vector_store_handler.handler, file_id)?;
    vector_store.insert_chunks(vector_store_handler.handler, &chunks)?;

    fs::remove_file(Path::new(&downloaded_file)).map_err(|err| err.to_string())?;

    Ok(())
}

fn to_plugin_index_id(index_id: Uuid) -> IndexId {
    index_id
        .as_bytes()
        .iter()
        .fold(2_166_136_261_u32, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
        })
        .wrapping_rem(i32::MAX as u32) as i32
        + 1
}

fn new_file_model(
    index_id: Uuid,
    file_info: &OutputFileInfo,
) -> Result<sqlite::models::file::ActiveModel, String> {
    Ok(sqlite::models::file::ActiveModel {
        index_id: Set(index_id.to_string()),
        name: Set(file_info.file_name.clone()),
        file_uri: Set(file_info.file_uri.clone()),
        file_path: Set(file_info.file_path.clone()),
        file_dir: Set(file_info.file_dir.clone()),
        file_name: Set(file_info.file_name.clone()),
        content_type: Set(file_info.content_type.clone()),
        size: Set(u64::try_from(file_info.size).map_err(|err| err.to_string())?),
        updated_at: Set(file_info.updated_at),
        file_hash: Set(file_info.content_hash.clone()),
        ..Default::default()
    })
}

fn set_file_info(
    file: &mut sqlite::models::file::ActiveModel,
    index_id: Uuid,
    file_info: &OutputFileInfo,
) -> Result<(), String> {
    file.index_id = Set(index_id.to_string());
    file.name = Set(file_info.file_name.clone());
    file.file_uri = Set(file_info.file_uri.clone());
    file.file_path = Set(file_info.file_path.clone());
    file.file_dir = Set(file_info.file_dir.clone());
    file.file_name = Set(file_info.file_name.clone());
    file.content_type = Set(file_info.content_type.clone());
    file.size = Set(u64::try_from(file_info.size).map_err(|err| err.to_string())?);
    file.updated_at = Set(file_info.updated_at);
    file.file_hash = Set(file_info.content_hash.clone());
    Ok(())
}
