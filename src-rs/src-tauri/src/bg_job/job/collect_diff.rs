use anydex_plugin_api_lib::file_source::types::InputFileInfo;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    bg_job::{
        entry::JobType,
        job::{process_file::ProcessFileArgs, Job, JobError},
    },
    plugin::{entry::FILE_SOURCE_MGR, file_source::interface::FileSource},
    repository::json::load,
    repository::sqlite::job::update_progress,
    usecase::{
        files::{delete_files, get_all_indexed_files},
        indexes::is_index_ready,
        jobs::{add_jobs, JobInfo},
        oauth::get_authentication,
    },
};

pub struct CollectDiff {
    db: DatabaseConnection,
}

impl CollectDiff {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectDiffArgs {
    pub index_id: Uuid,
}

impl Job for CollectDiff {
    type JobArgsType = CollectDiffArgs;
    async fn process(&self, job_id: i64, payload: Value) -> Result<(), JobError> {
        let args: CollectDiffArgs = serde_json::from_value(payload)
            .map_err(|err| JobError::non_retryable(err.to_string()))?;
        update_progress(&self.db, job_id, "インデックスを確認しています").await?;
        is_index_ready(args.index_id, &self.db).await?;
        let userdata = load()?;
        let index = userdata
            .indexes
            .iter()
            .find(|v| v.id == args.index_id)
            .ok_or_else(|| JobError::non_retryable("index not found"))?;
        let file_source = index
            .clone()
            .file_source
            .ok_or_else(|| JobError::non_retryable("file source not found"))?;
        let file_extractor = index
            .clone()
            .file_extractor
            .ok_or_else(|| JobError::non_retryable("file extractor not found"))?;
        let embedder = index
            .embedder
            .clone()
            .ok_or_else(|| JobError::non_retryable("embedder not found"))?;
        let vector_store = index
            .vector_store
            .clone()
            .ok_or_else(|| JobError::non_retryable("vector store not found"))?;
        let file_source_name = &file_source.name;

        let plugin = {
            let mut manager = FILE_SOURCE_MGR.lock().unwrap();
            let manager = manager.as_mut().ok_or_else(|| {
                JobError::non_retryable("file source plugin manager is not initialized")
            })?;
            manager
                .get_plugin(file_source_name)
                .ok_or_else(|| {
                    JobError::non_retryable(format!(
                        "File Source plugin {file_source_name} was not found in the plugin manager"
                    ))
                })?
                .clone()
        };
        let access_token = match &file_source.oauth_id {
            Some(v) => {
                let mut res = get_authentication(args.index_id, *v, &self.db)
                    .await?
                    .ok_or_else(|| JobError::non_retryable("oauth is set but not authenticated"))?;

                Some(res.load_access_token().await?)
            }
            None => None,
        };

        let files: Vec<InputFileInfo> = get_all_indexed_files(args.index_id, &self.db)
            .await?
            .iter()
            .map(|v| {
                Ok(InputFileInfo {
                    file_uri: v.file_uri.clone(),
                    size: usize::try_from(v.size).map_err(|err| err.to_string())?,
                    updated_at: v.updated_at,
                    content_hash: v.file_hash.clone(),
                })
            })
            .collect::<Result<_, String>>()?;

        let file_source_params = file_source.params.clone();
        update_progress(&self.db, job_id, "ファイルの差分を取得しています").await?;
        let diff = tokio::task::spawn_blocking(move || {
            let handler = plugin.get_handler(&file_source_params)?;
            plugin.get_diff(handler.handler, &files, access_token)
        })
        .await
        .map_err(|err| err.to_string())??;
        let deleted_count = diff.deleted.len();
        let added_or_updated = [diff.added, diff.updated].concat();
        update_progress(
            &self.db,
            job_id,
            format!(
                "差分を確認しました（追加・更新 {} 件、削除 {} 件）",
                added_or_updated.len(),
                deleted_count
            ),
        )
        .await?;
        update_progress(&self.db, job_id, "削除されたファイルを反映しています").await?;
        delete_files(args.index_id, &diff.deleted, &self.db).await?;

        let jobs = added_or_updated
            .iter()
            .map(|v| JobInfo {
                job_type: JobType::ProcessFile,
                args: serde_json::to_value(ProcessFileArgs {
                    index_id: args.index_id,
                    file_info: v.clone(),
                    file_source: file_source.clone(),
                    file_extractor: file_extractor.clone(),
                    embedder: embedder.clone(),
                    vector_store: vector_store.clone(),
                })
                .unwrap(),
            })
            .collect::<Vec<_>>();

        update_progress(&self.db, job_id, "追加・更新ファイルの処理を登録しています").await?;
        add_jobs(&self.db, args.index_id, jobs).await?;

        Ok(())
    }
}
