use anydex_plugin_api_lib::file_source::types::InputFileInfo;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    bg_job::{
        entry::JobType,
        job::{process_file::ProcessFileArgs, Job},
    },
    plugin::{entry::FILE_SOURCE_MGR, file_source::interface::FileSource},
    repository::json::load,
    usecase::{
        files::{delete_files, get_all_indexed_files},
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
    async fn process(&self, payload: Value) -> Result<(), String> {
        let args: CollectDiffArgs =
            serde_json::from_value(payload).map_err(|err| err.to_string())?;
        let userdata = load()?;
        let index = userdata
            .indexes
            .iter()
            .find(|v| v.id == args.index_id)
            .ok_or("index not found".to_string())?;
        let file_source = index.clone().file_source.ok_or("file source not found")?;
        let file_extractor = index
            .clone()
            .file_extractor
            .ok_or("file extractor not found")?;
        let embedder = index.embedder.clone().ok_or("embedder not found")?;
        let vector_store = index.vector_store.clone().ok_or("vector store not found")?;
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
        let access_token = match &file_source.oauth_id {
            Some(v) => {
                let mut res = get_authentication(args.index_id, *v, &self.db)
                    .await?
                    .ok_or("oauth is set but not authenticated".to_owned())?;

                Some(res.load_access_token().await?)
            }
            None => None,
        };

        let files: Vec<InputFileInfo> = get_all_indexed_files(args.index_id, &self.db)
            .await?
            .iter()
            .map(|v| InputFileInfo {
                file_uri: v.file_uri.clone(),
                size: v.size as usize,
                updated_at: v.updated_at,
                content_hash: v.file_hash.clone(),
            })
            .collect();

        let file_source_params = file_source.params.clone();
        let diff = tokio::task::spawn_blocking(move || {
            let handler = plugin.get_handler(&file_source_params)?;
            plugin.get_diff(handler.handler, &files, access_token)
        })
        .await
        .map_err(|err| err.to_string())??;
        delete_files(args.index_id, &diff.deleted, &self.db).await?;

        let added_or_updated = [diff.added, diff.updated].concat();
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

        add_jobs(&self.db, args.index_id, jobs).await?;

        Ok(())
    }
}
