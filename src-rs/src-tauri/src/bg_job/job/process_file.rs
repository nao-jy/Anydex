use anydex_plugin_api_lib::file_source::types::OutputFileInfo;
use anydex_plugin_api_lib::vector_store::types::{Chunk, FileId, IndexId};
use mime_guess::mime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::bg_job::entry::get_tmp_dir;
use crate::path;
use crate::{
    bg_job::job::Job,
    plugin::{
        embedder::interface::Embedder,
        entry::{EMBEDDER_MGR, FILE_EXTRACTOR_MGR, FILE_SOURCE_MGR, VECTOR_STORE_MGR},
        file_extractor::interface::FileExtractor,
        file_source::interface::FileSource,
        vector_store::interface::VectorStore,
    },
    repository::sqlite::job::update_progress,
    repository::{self, sqlite},
    usecase::oauth::get_authentication,
};

static NEW_FILE_LOCK: Mutex<()> = Mutex::const_new(());

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
    async fn process(&self, job_id: i64, payload: Value) -> Result<(), String> {
        let args: ProcessFileArgs =
            serde_json::from_value(payload).map_err(|err| err.to_string())?;

        update_progress(
            &self.db,
            job_id,
            format!("ファイル処理を準備しています: {}", args.file_info.file_name),
        )
        .await?;

        let file_entity = sqlite::models::file::Entity::find()
            .filter(sqlite::models::file::Column::IndexId.eq(args.index_id.to_string()))
            .filter(sqlite::models::file::Column::FileUri.eq(args.file_info.file_uri.clone()));
        let existing_file = file_entity
            .clone()
            .one(&self.db)
            .await
            .map_err(|err| err.to_string())?;

        if let Some(file) = existing_file {
            return self.update_existing_file(&args, file, job_id).await;
        }

        let _new_file_guard = NEW_FILE_LOCK.lock().await;
        let existing_file = file_entity
            .one(&self.db)
            .await
            .map_err(|err| err.to_string())?;
        if let Some(file) = existing_file {
            return self.update_existing_file(&args, file, job_id).await;
        }

        let plugin_index_id = to_plugin_index_id(args.index_id);
        let last_file = sqlite::models::file::Entity::find()
            .order_by_desc(sqlite::models::file::Column::Id)
            .one(&self.db)
            .await
            .map_err(|err| err.to_string())?;
        let file_id = last_file
            .map_or(Some(1), |file| file.id.checked_add(1))
            .ok_or_else(|| "file ID space exhausted".to_owned())?;
        let plugin_file_id = FileId::try_from(file_id).map_err(|err| err.to_string())?;
        process_file_plugins(&args, plugin_index_id, plugin_file_id, &self.db, job_id).await?;

        new_file_model(args.index_id, &args.file_info, file_id)?
            .insert(&self.db)
            .await
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

impl ProcessFile {
    async fn update_existing_file(
        &self,
        args: &ProcessFileArgs,
        file: sqlite::models::file::Model,
        job_id: i64,
    ) -> Result<(), String> {
        let file_id = FileId::try_from(file.id).map_err(|err| err.to_string())?;
        process_file_plugins(
            args,
            to_plugin_index_id(args.index_id),
            file_id,
            &self.db,
            job_id,
        )
        .await?;

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
    job_id: i64,
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
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();
    let mut process_task = tokio::task::spawn_blocking(move || {
        process_file_plugins_sync(
            &args,
            index_id,
            file_id,
            oauth_token,
            &downloaded_dir,
            &extracted_dir_string,
            &progress_tx,
        )
    });
    let mut progress_error = None;
    let result = loop {
        tokio::select! {
            result = &mut process_task => {
                break result.map_err(|err| err.to_string())?;
            }
            progress = progress_rx.recv() => {
                if let Some(progress) = progress {
                    if progress_error.is_none() {
                        progress_error = update_progress(db, job_id, progress).await.err();
                    }
                }
            }
        }
    };
    while let Ok(progress) = progress_rx.try_recv() {
        if progress_error.is_none() {
            progress_error = update_progress(db, job_id, progress).await.err();
        }
    }
    if let Some(error) = progress_error {
        return Err(format!("failed to update job progress: {error}"));
    }
    result
}

fn process_file_plugins_sync(
    args: &ProcessFileArgs,
    index_id: IndexId,
    file_id: FileId,
    oauth_token: Option<String>,
    downloaded_dir: &str,
    extracted_dir_string: &str,
    progress_tx: &mpsc::UnboundedSender<String>,
) -> Result<(), String> {
    let file_name = &args.file_info.file_name;
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
    let vector_store_handler = vector_store.get_handler(
        &args.vector_store.params,
        index_id.clone(),
        &path::get_dir().app_data_dir.to_string_lossy().to_string(),
    )?;

    send_progress(progress_tx, format!("ダウンロード中: {file_name}"))?;
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

    send_progress(progress_tx, format!("ファイルを展開中: {file_name}"))?;
    let extracted_files =
        file_extractor.extract(file_extractor_handler.handler, &downloaded_file)?;
    let mut chunks = Vec::with_capacity(extracted_files.len());
    for (_size, relative_path) in extracted_files.iter().enumerate() {
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
        send_progress(
            progress_tx,
            format!(
                "埋め込みを作成中: {} ({}/{})",
                file_name,
                chunks.len() + 1,
                extracted_files.len()
            ),
        )?;
        let content_vector = embedder.embed(
            embedder_handler.handler,
            extracted_file
                .to_str()
                .ok_or_else(|| "extracted file path is not valid UTF-8".to_owned())?,
        )?;
        chunks.push(Chunk {
            id: Uuid::now_v7().to_string(),
            file_id,
            index_id: index_id.clone(),
            content,
            content_vector,
        });
        fs::remove_file(extracted_file).map_err(|err| err.to_string())?;
    }

    send_progress(progress_tx, format!("ベクトルストアを更新中: {file_name}"))?;
    vector_store.delete_chunks_file(vector_store_handler.handler, file_id)?;
    vector_store.insert_chunks(vector_store_handler.handler, &chunks)?;

    fs::remove_file(Path::new(&downloaded_file)).map_err(|err| err.to_string())?;

    Ok(())
}

fn send_progress(
    progress_tx: &mpsc::UnboundedSender<String>,
    progress_info: String,
) -> Result<(), String> {
    progress_tx
        .send(progress_info)
        .map_err(|err| err.to_string())
}

fn to_plugin_index_id(index_id: Uuid) -> IndexId {
    index_id.to_string()
}

fn new_file_model(
    index_id: Uuid,
    file_info: &OutputFileInfo,
    id: i64,
) -> Result<sqlite::models::file::ActiveModel, String> {
    Ok(sqlite::models::file::ActiveModel {
        id: Set(id),
        index_id: Set(index_id.to_string()),
        name: Set(file_info.file_name.clone()),
        file_uri: Set(file_info.file_uri.clone()),
        file_path: Set(file_info.file_path.clone()),
        file_dir: Set(file_info.file_dir.clone()),
        file_name: Set(file_info.file_name.clone()),
        content_type: Set(file_info.content_type.clone()),
        size: Set(i64::try_from(file_info.size).map_err(|err| err.to_string())?),
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
    file.size = Set(i64::try_from(file_info.size).map_err(|err| err.to_string())?);
    file.updated_at = Set(file_info.updated_at);
    file.file_hash = Set(file_info.content_hash.clone());
    Ok(())
}
