use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::Path,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::Handler,
    file_source::{
        params::*,
        types::{FileIterationInfo, InputFileInfo, OutputFileInfo},
    },
};
use chrono::DateTime;
use opendal::{Entry, Operator};
use schemars::schema_for;
use tokio::runtime::Runtime;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    gdrive::{export_gdrive_file, native_export_format},
    param::Param,
};

struct HandleInfo {
    pub last_indexed_files: HashMap<String, InputFileInfo>,
    pub download_dir: String,
    pub rt: Runtime,
    pub entries: VecDeque<Entry>,
    pub delete_entries: HashSet<String>,
    pub operator: Operator,
    pub gdrive_access_token: Option<String>,
    pub gdrive_root: String,
}

static HANDLERS: LazyLock<Mutex<HashMap<Handler, HandleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLER_COUNTER: Mutex<u64> = Mutex::new(0);

pub fn get_param_schema(_args: GetParamSchemaArgs) -> Result<GetParamSchemaData, String> {
    let schema = schema_for!(Param);
    return Ok(GetParamSchemaData {
        schema: serde_json::to_value(&schema).map_err(|err| err.to_string())?,
    });
}

fn config_value_to_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => Some(String::new()),
        serde_json::Value::Bool(_) | serde_json::Value::Number(_) => Some(value.to_string()),
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| match value {
                serde_json::Value::Bool(_)
                | serde_json::Value::Number(_)
                | serde_json::Value::String(_) => Some(
                    value
                        .as_str()
                        .map_or_else(|| value.to_string(), str::to_owned),
                ),
                serde_json::Value::Null
                | serde_json::Value::Array(_)
                | serde_json::Value::Object(_) => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|values| values.join(",")),
        serde_json::Value::Object(_) => None,
    }
}

fn service_config_to_args(
    param: &Param,
    access_token: Option<&str>,
) -> Result<Vec<(String, String)>, String> {
    let mut args = param
        .service_configurations
        .iter()
        .map(|(key, value)| {
            config_value_to_string(value)
                .map(|value| (key.clone(), value))
                .ok_or_else(|| format!("invalid service configuration value for '{key}'"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(access_token) = access_token {
        let oauth_name = param
            .oauth_conf_access_token_name
            .as_deref()
            .unwrap_or("access_token");
        if let Some((_, value)) = args.iter_mut().find(|(key, _)| key == oauth_name) {
            value.clear();
            value.push_str(access_token);
        } else {
            args.push((oauth_name.to_owned(), access_token.to_owned()));
        }
    }

    Ok(args)
}

fn rename_to_uuid(filename: &str) -> String {
    let path = Path::new(filename);

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    format!("{}{}", Uuid::new_v4(), ext)
}

pub fn get_handler(args: GetHandlerArgs) -> Result<GetHandlerData, String> {
    let param = serde_json::from_value::<Param>(args.params).map_err(|err| err.to_string())?;
    let access_token = args.access_token;
    let config_args = service_config_to_args(&param, access_token.as_deref())?;

    let files_vec = args.last_indexed_files;

    let gdrive = param.service_name.eq_ignore_ascii_case("gdrive");
    let gdrive_root = param
        .service_configurations
        .get("root")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let oauth_name = param
        .oauth_conf_access_token_name
        .as_deref()
        .unwrap_or("access_token");
    let gdrive_access_token = if gdrive {
        access_token.clone().or_else(|| {
            param
                .service_configurations
                .get(oauth_name)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
    } else {
        None
    };
    let operator =
        Operator::via_iter(param.service_name, config_args).map_err(|err| err.to_string())?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| err.to_string())?;
    let entries =
        match rt.block_on(async { operator.list_with("").recursive(param.is_recursive).await }) {
            Ok(entries) => Ok(entries.into()),
            Err(err) => Err(err.to_string()),
        }?;

    let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
    let handler = (*handler_mtx).clone();
    (*handler_mtx) += 1;
    let info = HandleInfo {
        last_indexed_files: files_vec
            .iter()
            .map(|value| (value.file_id.clone(), value.clone()))
            .collect(),
        download_dir: args.download_dir,
        entries,
        operator,
        rt,
        delete_entries: files_vec.iter().map(|v| v.file_id.clone()).collect(),
        gdrive_access_token,
        gdrive_root,
    };

    HANDLERS.lock().unwrap().insert(handler, info);
    Ok(GetHandlerData { handler })
}

pub fn free_handler(args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
    HANDLERS.lock().unwrap().remove(&args.handler);
    Ok(FreeHandlerData {})
}

pub fn next(args: NextArgs) -> Result<NextData, String> {
    let handler = args.handler;
    let mut mtx = HANDLERS.lock().unwrap();
    let info = mtx
        .get_mut(&handler)
        .ok_or(format!("handler {handler} not found"))?;

    let last_indexed_files = &info.last_indexed_files;
    let delete_entries = &mut info.delete_entries;
    let dl_dir = &info.download_dir.clone();
    let operator = &mut info.operator;
    let rt = &info.rt;

    while let Some(entry) = info.entries.pop_front() {
        if entry.metadata().mode().is_dir() {
            continue;
        }

        let uri = entry.path();
        let last_modified = entry.metadata().last_modified().map(|value| {
            let jiff = value.into_inner();
            DateTime::from_timestamp(jiff.as_second(), jiff.subsec_nanosecond() as u32).unwrap()
        });
        let mut size = entry.metadata().content_length() as usize;
        let mut name = entry.name().to_owned();
        let mut content_type = entry.metadata().content_type().map_or(
            mime_guess::from_path(&name)
                .first_or_octet_stream()
                .to_string(),
            |f| f.to_string(),
        );
        let is_gdrive_native = native_export_format(&content_type).is_some();
        let mut prev_file_hash: Option<u64> = None;

        delete_entries.remove(uri);

        let prev_entry = last_indexed_files.get(uri);

        // added
        let added_or_updated = match prev_entry {
            Some(prev_entry_unwrapped) => {
                prev_file_hash = Some(prev_entry_unwrapped.file_hash);
                (!is_gdrive_native && prev_entry_unwrapped.size != size)
                    || last_modified
                        .is_none_or(|v| prev_entry_unwrapped.updated_at.is_none_or(|vv| vv < v))
            }
            None => true,
        };
        if added_or_updated {
            let file_blob_content = if is_gdrive_native {
                let Some(access_token) = info.gdrive_access_token.as_deref() else {
                    return Err(
                        "Google Drive native document export requires an OAuth access token".into(),
                    );
                };
                rt.block_on(export_gdrive_file(
                    access_token,
                    &info.gdrive_root,
                    uri,
                    &content_type,
                ))
                .map(|(content, exported_content_type, exported_name)| {
                    size = content.len();
                    content_type = exported_content_type;
                    name = exported_name;
                    content
                })?
            } else {
                rt.block_on(async { operator.read(uri).await })
                    .map(|v| v.to_vec())
                    .map_err(|err| err.to_string())?
            };

            // check file hash
            let file_hash = xxh3_64(&file_blob_content);
            if prev_file_hash == Some(file_hash) {
                continue;
            }

            let dl_filename = rename_to_uuid(&name);
            let download_path = Path::new(dl_dir)
                .join(dl_filename)
                .to_string_lossy()
                .to_string();

            fs::create_dir_all(dl_dir).map_err(|err| err.to_string())?;

            let added = OutputFileInfo {
                file_id: uri.to_string(),
                name: name,
                downloaded_file_path: download_path,
                content_type: content_type,
                size: size,
                updated_at: last_modified,
                metadata: serde_json::Value::Null,
                file_hash: file_hash,
            };
            return Ok(NextData {
                info: Some(FileIterationInfo::AddedOrUpdated(added)),
            });
        }
    }
    if let Some(entry) = delete_entries.iter().next().cloned() {
        delete_entries.remove(&entry);
        return Ok(NextData {
            info: Some(FileIterationInfo::Deleted(entry)),
        });
    }
    return Ok(NextData { info: None });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_configurations_are_flattened_and_oauth_token_overrides_config() {
        let param = serde_json::from_value(serde_json::json!({
            "service_name": "gdrive",
            "service_configurations": {
                "root": "/documents",
                "access_token": "stale-token"
            },
            "oauth_conf_access_token_name": null
        }))
        .unwrap();

        let args = service_config_to_args(&param, Some("fresh-token")).unwrap();

        assert_eq!(
            args,
            vec![
                ("root".to_owned(), "/documents".to_owned()),
                ("access_token".to_owned(), "fresh-token".to_owned())
            ]
        );
    }

    #[test]
    fn service_configurations_reject_nested_objects() {
        let param = serde_json::from_value(serde_json::json!({
            "service_name": "gdrive",
            "service_configurations": {
                "root": { "unexpected": "object" }
            },
            "oauth_conf_access_token_name": null
        }))
        .unwrap();

        assert!(service_config_to_args(&param, None).is_err());
    }

    #[test]
    fn google_workspace_files_use_office_export_formats() {
        assert_eq!(
            native_export_format("application/vnd.google-apps.document"),
            Some((
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "docx"
            ))
        );
        assert_eq!(
            native_export_format("application/vnd.google-apps.spreadsheet"),
            Some((
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "xlsx"
            ))
        );
        assert_eq!(
            native_export_format("application/vnd.google-apps.presentation"),
            Some((
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                "pptx"
            ))
        );
        assert_eq!(native_export_format("application/pdf"), None);
    }
}
