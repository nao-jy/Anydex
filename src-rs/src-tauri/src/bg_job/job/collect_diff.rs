use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    bg_job::job::Job,
    plugin::{entry::FILE_SOURCE_MGR, file_source::interface::FileSource},
    repository,
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
    fn process(&self, payload: Value) -> Result<(), String> {
        let args: CollectDiffArgs =
            serde_json::from_value(payload).map_err(|err| err.to_string())?;
        let file_source_name = &args.file_source.name;

        let plugin = {
            let mut manager = FILE_SOURCE_MGR.lock().unwrap();
            let manager = manager
                .as_mut()
                .ok_or_else(|| "file source plugin manager is not initialized".to_owned())?;
            manager
                .get_plugin(file_source_name)
                .ok_or_else(|| {
                    format!(
                        "File Source plugin {file_source_name} was not found in the plugin manager"
                    )
                })?
                .clone()
        };
        let handler = plugin.get_handler(&args.file_source.params)?;
        plugin.get_diff(handler.handler, last_indexed_files, access_token);

        Ok(())
    }
}
