use std::{
    fs::{self, File},
    io::BufWriter,
};

use crate::{path::get_dir, repository::json::models::UserData};

pub mod models;
pub mod oauth_client;

fn json_path() -> Result<String, String> {
    let app_data_dir = get_dir().app_data_dir;
    let json_path = app_data_dir
        .join("userdata.json")
        .to_string_lossy()
        .to_string();
    Ok(json_path)
}

pub fn save(userdata: &UserData) -> Result<(), String> {
    let json_path = json_path()?;
    let file = File::create(json_path).map_err(|err| err.to_string())?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, userdata).map_err(|err| err.to_string())?;
    Ok(())
}

pub fn load() -> Result<UserData, String> {
    let json_path = json_path()?;
    let data = fs::read_to_string(json_path);
    match data {
        Ok(s) => {
            let mut userdata = match serde_json::from_str(&s) {
                Ok(userdata) => userdata,
                Err(_) => UserData::default(),
            };
            normalize_plugin_params(&mut userdata);
            Ok(userdata)
        }
        Err(_) => Ok(UserData::default()),
    }
}

fn normalize_plugin_params(userdata: &mut UserData) {
    fn normalize(params: &mut serde_json::Value) {
        if let serde_json::Value::String(serialized) = params {
            if let Ok(parsed) = serde_json::from_str(serialized) {
                *params = parsed;
            }
        }
    }

    for index in &mut userdata.indexes {
        if let Some(file_source) = &mut index.file_source {
            normalize(&mut file_source.params);
        }
        if let Some(file_extractor) = &mut index.file_extractor {
            normalize(&mut file_extractor.params);
        }
        if let Some(embedder) = &mut index.embedder {
            normalize(&mut embedder.params);
        }
        if let Some(vector_store) = &mut index.vector_store {
            normalize(&mut vector_store.params);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::repository::json::models::{FileSource, Index, UserData};

    use super::normalize_plugin_params;

    #[test]
    fn normalizes_legacy_stringified_plugin_params() {
        let mut userdata = UserData {
            indexes: vec![Index {
                file_source: Some(FileSource {
                    name: "gdrive".to_owned(),
                    params: serde_json::Value::String(
                        r#"{"service_name":"gdrive","service_configurations":{},"is_recursive":true}"#
                            .to_owned(),
                    ),
                    oauth_id: None,
                }),
                ..Index::default()
            }],
            ..UserData::default()
        };

        normalize_plugin_params(&mut userdata);

        assert_eq!(
            userdata.indexes[0].file_source.as_ref().unwrap().params,
            json!({
                "service_name": "gdrive",
                "service_configurations": {},
                "is_recursive": true
            })
        );
    }
}
