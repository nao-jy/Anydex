use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::Handler,
    file_source::{
        params::*,
        types::{InputFileInfo, OutputFileInfo},
    },
};
use chrono::DateTime;
use opendal::Operator;
use schemars::schema_for;
use uuid::Uuid;

use crate::{
    gdrive::{export_gdrive_file, native_export_format},
    param::Param,
};

struct HandleInfo {
    pub param: Param,
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

fn uri_to_path_dir_name(uri: &str) -> Result<(String, String, String), String> {
    if uri.is_empty() || uri.starts_with('/') {
        return Err(format!("invalid storage URI: {uri}"));
    }

    let (parent, file_name) = uri.rsplit_once('/').unwrap_or(("", uri));
    if file_name.is_empty() {
        return Err(format!("storage URI has no file name: {uri}"));
    }

    let file_path = format!("/{uri}");
    let file_dir = if parent.is_empty() {
        "/".to_owned()
    } else {
        format!("/{parent}")
    };

    Ok((file_path, file_dir, file_name.to_owned()))
}

pub fn get_handler(args: GetHandlerArgs) -> Result<GetHandlerData, String> {
    let param = serde_json::from_value::<Param>(args.params).map_err(|err| err.to_string())?;

    let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
    let handler = (*handler_mtx).clone();
    (*handler_mtx) += 1;
    let info = HandleInfo { param };

    HANDLERS.lock().unwrap().insert(handler, info);
    Ok(GetHandlerData { handler })
}

pub fn free_handler(args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
    HANDLERS.lock().unwrap().remove(&args.handler);
    Ok(FreeHandlerData {})
}

pub fn get_diff(args: GetDiffArgs) -> Result<GetDiffData, String> {
    let handler = args.handler;
    let mut mtx = HANDLERS.lock().unwrap();
    let info = mtx
        .get_mut(&handler)
        .ok_or(format!("handler {handler} not found"))?;

    let param = info.param.clone();
    let access_token = args.access_token;
    let config_args = service_config_to_args(&param, access_token.as_deref())?;

    let operator =
        Operator::via_iter(param.service_name, config_args).map_err(|err| err.to_string())?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| err.to_string())?;
    let entries = rt
        .block_on(async { operator.list_with("").recursive(param.is_recursive).await })
        .map_err(|err| err.to_string())?;

    let mut delete_uris: HashSet<String> = args
        .last_indexed_files
        .iter()
        .map(|v| v.file_uri.to_string())
        .collect();

    let mut added_or_updated_entries: Vec<OutputFileInfo> = Vec::new();

    let last_indexed_files = HashMap::<String, InputFileInfo>::from_iter(
        args.last_indexed_files
            .iter()
            .map(|v| (v.file_uri.to_string(), v.clone())),
    );

    for entry in entries {
        if entry.metadata().mode().is_dir() {
            continue;
        }
        let uri = entry.path();
        delete_uris.remove(uri);

        let (file_path, file_dir, file_name) = uri_to_path_dir_name(uri)?;
        let last_modified = entry.metadata().last_modified().map(|value| {
            let jiff = value.into_inner();
            DateTime::from_timestamp(jiff.as_second(), jiff.subsec_nanosecond() as u32).unwrap()
        });
        let size = entry.metadata().content_length() as usize;
        let content_type = entry.metadata().content_type().map_or(
            mime_guess::from_path(&file_name)
                .first_or_octet_stream()
                .to_string(),
            |f| f.to_string(),
        );

        let prev_entry = last_indexed_files.get(&file_path);

        // added
        let added_or_updated = match prev_entry {
            Some(prev_entry_unwrapped) => {
                (prev_entry_unwrapped.size != size)
                    || last_modified
                        .is_none_or(|v| prev_entry_unwrapped.updated_at.is_none_or(|vv| vv < v))
            }
            None => true,
        };
        if added_or_updated {
            let added = OutputFileInfo {
                file_uri: uri.to_string(),
                file_path,
                file_dir,
                file_name: file_name,
                content_type: content_type,
                size: size,
                updated_at: last_modified,
                metadata: serde_json::Value::Null,
            };
            added_or_updated_entries.push(added);
        }
    }

    return Ok(GetDiffData {
        added_or_updated: added_or_updated_entries,
        deleted: delete_uris.iter().map(|v| v.clone()).collect(),
    });
}

pub fn download(args: DownloadArgs) -> Result<DownloadData, String> {
    let handler = args.handler;
    let mut mtx = HANDLERS.lock().unwrap();
    let info = mtx
        .get_mut(&handler)
        .ok_or(format!("handler {handler} not found"))?;

    let param = info.param.clone();
    let access_token = args.access_token;
    let config_args = service_config_to_args(&param, access_token.as_deref())?;
    let uri = args.uri;

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
    let metadata = rt
        .block_on(async { operator.stat(&uri).await })
        .map_err(|err| err.to_string())?;

    if metadata.mode().is_dir() {
        return Err(format!("{uri} is not file"));
    }

    let (_file_path, _file_dir, mut file_name) = uri_to_path_dir_name(&uri)?;
    let content_type = metadata.content_type().map_or(
        mime_guess::from_path(&file_name)
            .first_or_octet_stream()
            .to_string(),
        |f| f.to_string(),
    );
    let is_gdrive_native = native_export_format(&content_type).is_some();

    let file_blob_content = if is_gdrive_native {
        let Some(access_token) = gdrive_access_token.as_deref() else {
            return Err(
                "Google Drive native document export requires an OAuth access token".into(),
            );
        };
        rt.block_on(export_gdrive_file(
            access_token,
            &gdrive_root,
            &uri,
            &content_type,
        ))
        .map(|(content, _exported_content_type, exported_name)| {
            file_name = exported_name;
            content
        })?
    } else {
        rt.block_on(async { operator.read(&uri).await })
            .map(|v| v.to_vec())
            .map_err(|err| err.to_string())?
    };

    let dst_path = Path::new(&args.dst_dir).join(rename_to_uuid(&file_name));

    fs::write(&dst_path, file_blob_content).map_err(|err| err.to_string())?;

    return Ok(DownloadData {
        downloaded_file_path: dst_path.to_string_lossy().to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_is_converted_to_pseudo_path_directory_and_file_name() {
        assert_eq!(
            uri_to_path_dir_name("hoge/fuga/file.ext").unwrap(),
            (
                "/hoge/fuga/file.ext".to_owned(),
                "/hoge/fuga".to_owned(),
                "file.ext".to_owned()
            )
        );
        assert_eq!(
            uri_to_path_dir_name("file.ext").unwrap(),
            (
                "/file.ext".to_owned(),
                "/".to_owned(),
                "file.ext".to_owned()
            )
        );
    }

    #[test]
    fn uri_without_a_file_name_is_rejected() {
        assert!(uri_to_path_dir_name("").is_err());
        assert!(uri_to_path_dir_name("hoge/fuga/").is_err());
        assert!(uri_to_path_dir_name("/hoge/fuga/file.ext").is_err());
    }

    #[test]
    fn service_configurations_are_flattened_and_oauth_token_overrides_config() {
        let param = serde_json::from_value(serde_json::json!({
            "service_name": "gdrive",
            "service_configurations": {
                "root": "/documents",
                "access_token": "stale-token"
            },
            "oauth_conf_access_token_name": null,
            "is_recursive": false
        }))
        .unwrap();

        let args = service_config_to_args(&param, Some("fresh-token")).unwrap();

        assert_eq!(
            args,
            vec![
                ("root".to_owned(), "/documents".to_owned()),
                ("access_token".to_owned(), "fresh-token".to_owned()),
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
            "oauth_conf_access_token_name": null,
            "is_recursive": false
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
