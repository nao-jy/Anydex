use anydex_plugin_api_lib::file_source::types::InputFileInfo;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    bg_job::job::Job,
    plugin::{entry::FILE_SOURCE_MGR, file_source::interface::FileSource},
    repository,
    usecase::{
        files::{delete_files, get_all_indexed_files},
        oauth::get_authentication,
    },
};

pub struct CollectDiff {
    db: DatabaseConnection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectDiffArgs {
    pub index_id: Uuid,
    pub file_source: repository::json::models::FileSource,
}

impl Job for CollectDiff {
    async fn process(&self, payload: Value) -> Result<(), String> {
        let args: CollectDiffArgs =
            serde_json::from_value(payload).map_err(|err| err.to_string())?;
        let file_source_name = &args.file_source.name;

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
        let handler = plugin.get_handler(&args.file_source.params)?;
        let access_token = match args.file_source.oauth_id {
            Some(v) => {
                let mut res = get_authentication(args.index_id, v, &self.db)
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

        let diff = plugin.get_diff(handler.handler, &files, access_token)?;
        delete_files(args.index_id, &diff.deleted, &self.db).await?;

        todo!("dispatch file process");

        Ok(())
    }
}
