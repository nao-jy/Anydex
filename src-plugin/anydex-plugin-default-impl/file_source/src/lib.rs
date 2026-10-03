use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::{c_char, c_void, CString},
    fs,
    path::Path,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{to_string, CResult, CSlice, Handler, LogLevel},
    file_source::interface::{
        CFileIterationInfo, CInputFileInfo, COutputFileInfo, DynamicFileSourceV1,
        DYNAMIC_FILE_SOURCE_ABI_VERSION_V1, INVALID_HANDLE,
    },
    interface::PluginInit,
};
use opendal::{Entry, Operator};
use schemars::schema_for;
use tokio::runtime::Runtime;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use crate::param::Param;

pub mod param;

static PLUGIN: Mutex<Option<DynamicFileSourceV1>> = Mutex::new(None);
static PLUGIN_INIT: Mutex<Option<PluginInit>> = Mutex::new(None);

static mut LOGGER: Option<unsafe extern "C" fn(log_level: LogLevel, message: *const c_char)> = None;

fn log(level: LogLevel, msg: &str) {
    let logger = unsafe { std::ptr::read_volatile(std::ptr::addr_of!(LOGGER)) };
    let Some(logger_fn) = logger else {
        return;
    };

    let cmsg = CString::new(msg).unwrap();
    unsafe { logger_fn(level, cmsg.as_ptr()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_plugin(init: *const PluginInit) -> *const DynamicFileSourceV1 {
    let plugin_read = unsafe { init.read().clone() };
    unsafe {
        std::ptr::write(std::ptr::addr_of_mut!(LOGGER), Some(plugin_read.log));
    }

    let mut plugin_mtx = PLUGIN.lock().unwrap();
    if (*plugin_mtx).is_none() {
        let mut plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
        *plugin_init_mtx = Some(plugin_read);

        let new_plugin = DynamicFileSourceV1 {
            abi_version: DYNAMIC_FILE_SOURCE_ABI_VERSION_V1,
            struct_size: std::mem::size_of::<DynamicFileSourceV1>(),
            get_param_schema,
            get_handler,
            free_handler,
            next,
            free,
        };
        *plugin_mtx = Some(new_plugin);
    }

    let ret = plugin_mtx.as_ref().unwrap();
    return &raw const *ret;
}

#[derive(Debug, Clone)]
struct InputFileInfo {
    pub file_id: String,
    pub size: usize,
    pub updated_at_utc: i64,
    pub file_hash: u64,
}
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

static STR_REF: LazyLock<Mutex<HashMap<usize, Box<CString>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLERS: LazyLock<Mutex<HashMap<Handler, HandleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

static HANDLER_COUNTER: Mutex<u64> = Mutex::new(0);

unsafe extern "C" fn get_param_schema() -> *const c_char {
    let schema = schema_for!(Param);
    let schema_str: Box<CString> =
        Box::from(CString::new(serde_json::to_string_pretty(&schema).unwrap()).unwrap());
    let ptr = schema_str.as_ptr();
    STR_REF.lock().unwrap().insert(ptr as usize, schema_str);

    return ptr;
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

fn native_export_format(content_type: &str) -> Option<(&'static str, &'static str)> {
    match content_type {
        "application/vnd.google-apps.document" => Some((
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "docx",
        )),
        "application/vnd.google-apps.spreadsheet" => Some((
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xlsx",
        )),
        "application/vnd.google-apps.presentation" => Some((
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "pptx",
        )),
        _ => None,
    }
}

#[derive(serde::Deserialize)]
struct GdriveFileId {
    id: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(serde::Deserialize)]
struct GdriveFileIdList {
    files: Vec<GdriveFileId>,
}

async fn find_gdrive_file_id(access_token: &str, root: &str, path: &str) -> Result<String, String> {
    let segments: Vec<_> = root
        .split('/')
        .chain(path.split('/'))
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Err("cannot export a Google Drive file with an empty path".into());
    }

    let mut parent_id = "root".to_owned();
    for (index, segment) in segments.iter().enumerate() {
        let escaped_name = segment.replace('\\', "\\\\").replace('\'', "\\'");
        let query =
            format!("name = '{escaped_name}' and '{parent_id}' in parents and trashed = false");
        let response = HTTP_CLIENT
            .get("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(access_token)
            .query(&[
                ("q", query.as_str()),
                ("pageSize", "1000"),
                ("orderBy", "modifiedTime desc,createdTime desc"),
                ("fields", "files(id,mimeType)"),
            ])
            .send()
            .await
            .map_err(|err| format!("Google Drive file lookup failed: {err}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Google Drive file lookup returned {status}: {body}"
            ));
        }
        let files: GdriveFileIdList = response
            .json()
            .await
            .map_err(|err| format!("invalid Google Drive file lookup response: {err}"))?;
        let file = files
            .files
            .into_iter()
            .next()
            .ok_or_else(|| format!("Google Drive path segment '{segment}' was not found"))?;

        if index + 1 < segments.len() {
            if file.mime_type != "application/vnd.google-apps.folder" {
                return Err(format!(
                    "Google Drive path segment '{segment}' is not a folder"
                ));
            }
            parent_id = file.id;
        } else {
            return Ok(file.id);
        }
    }

    Err("Google Drive file path could not be resolved".into())
}

async fn export_gdrive_file(
    access_token: &str,
    root: &str,
    path: &str,
    content_type: &str,
) -> Result<(Vec<u8>, String, String), String> {
    let (export_mime_type, extension) = native_export_format(content_type)
        .ok_or_else(|| format!("unsupported Google Drive native file type: {content_type}"))?;
    let file_id = find_gdrive_file_id(access_token, root, path).await?;
    let url = format!("https://www.googleapis.com/drive/v3/files/{file_id}/export");
    let response = HTTP_CLIENT
        .get(url)
        .bearer_auth(access_token)
        .query(&[("mimeType", export_mime_type)])
        .send()
        .await
        .map_err(|err| format!("Google Drive export request failed: {err}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Google Drive export returned {status}: {body}"));
    }
    let content = response
        .bytes()
        .await
        .map_err(|err| format!("failed to read Google Drive export: {err}"))?
        .to_vec();
    let filename = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path);
    let exported_name = Path::new(filename)
        .with_extension(extension)
        .to_string_lossy()
        .into_owned();

    Ok((content, export_mime_type.to_owned(), exported_name))
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

unsafe extern "C" fn get_handler(
    params: *const c_char,
    last_indexed_files: CSlice<CInputFileInfo>,
    access_token: *const c_char,
    download_dir: *const c_char,
) -> Handler {
    let param_str = to_string(params);
    let param = match serde_json::from_str::<Param>(&param_str) {
        Ok(param) => param,
        Err(err) => {
            log(LogLevel::Error, &err.to_string());
            return INVALID_HANDLE;
        }
    };
    let access_token = (!access_token.is_null()).then(|| to_string(access_token));
    let args = match service_config_to_args(&param, access_token.as_deref()) {
        Ok(args) => args,
        Err(err) => {
            log(LogLevel::Error, &err);
            return INVALID_HANDLE;
        }
    };

    let cfiles_vec: Vec<CInputFileInfo> = last_indexed_files.into();
    let files_vec: Vec<InputFileInfo> = cfiles_vec
        .iter()
        .map(|f| InputFileInfo {
            file_id: to_string(f.file_id),
            size: f.size,
            updated_at_utc: f.updated_at_utc,
            file_hash: f.file_hash,
        })
        .collect();

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
    let operator = match Operator::via_iter(param.service_name, args) {
        Ok(operator) => operator,
        Err(err) => {
            log(LogLevel::Error, &err.to_string());
            return INVALID_HANDLE;
        }
    };
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            log(LogLevel::Error, &err.to_string());
            return INVALID_HANDLE;
        }
    };
    let entries =
        match rt.block_on(async { operator.list_with("").recursive(param.is_recursive).await }) {
            Ok(entries) => entries.into(),
            Err(err) => {
                log(LogLevel::Error, &err.to_string());
                return INVALID_HANDLE;
            }
        };

    let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
    let handler = (*handler_mtx).clone();
    (*handler_mtx) += 1;
    let info = HandleInfo {
        last_indexed_files: files_vec
            .iter()
            .map(|value| (value.file_id.clone(), value.clone()))
            .collect(),
        download_dir: to_string(download_dir),
        entries,
        operator,
        rt,
        delete_entries: files_vec.iter().map(|v| v.file_id.clone()).collect(),
        gdrive_access_token,
        gdrive_root,
    };

    HANDLERS.lock().unwrap().insert(handler, info);
    handler
}

unsafe extern "C" fn free_handler(handler: Handler) {
    HANDLERS.lock().unwrap().remove(&handler);
}

struct NextRef {
    pub result: Box<CResult<CFileIterationInfo>>,
    pub added_or_updated: Option<Box<COutputFileInfo>>,
    pub chars: Vec<Box<CString>>,
}

static NEXT_REF: LazyLock<Mutex<HashMap<usize, NextRef>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

unsafe impl Send for NextRef {}
unsafe impl Sync for NextRef {}

unsafe extern "C" fn next(handler: Handler) -> *const CResult<CFileIterationInfo> {
    let mut mtx = HANDLERS.lock().unwrap();
    let info_opt = mtx.get_mut(&handler);
    let mut nex_ref = NextRef {
        result: Box::new(CResult {
            success: false,
            data: CFileIterationInfo {
                added_or_updated: std::ptr::null(),
                deleted: std::ptr::null(),
            },
        }),
        added_or_updated: None,
        chars: Vec::new(),
    };

    if let Some(info) = info_opt {
        let last_indexed_files = &info.last_indexed_files;
        let delete_entries = &mut info.delete_entries;
        let dl_dir = &info.download_dir.clone();
        let operator = &mut info.operator;
        let rt = &info.rt;

        let mut failed = false;
        while let Some(entry) = info.entries.pop_front() {
            if entry.metadata().mode().is_dir() {
                continue;
            }

            let uri = entry.path();
            let last_modified = entry
                .metadata()
                .last_modified()
                .map_or(-1, |value| value.into_inner().as_second());
            let mut size = entry.metadata().content_length() as usize;
            let mut name = entry.name().to_owned();
            let mut content_type = entry.metadata().content_type().map_or(
                mime_guess::from_path(&name)
                    .first_or_octet_stream()
                    .to_string(),
                |f| f.to_string(),
            );
            let metadata = serde_json::Value::Null.to_string();
            let is_gdrive_native = native_export_format(&content_type).is_some();
            let mut prev_file_hash: Option<u64> = None;

            delete_entries.remove(uri);

            let prev_entry = last_indexed_files.get(uri);

            // added
            let added_or_updated = if prev_entry.is_none() {
                true
            } else {
                // updated
                let prev_entry_unwrapped = prev_entry.unwrap();
                prev_file_hash = Some(prev_entry_unwrapped.file_hash);
                (!is_gdrive_native && prev_entry_unwrapped.size != size)
                    || (prev_entry_unwrapped.updated_at_utc < last_modified || last_modified < 0)
            };
            if added_or_updated {
                let file_blob_content = if is_gdrive_native {
                    let Some(access_token) = info.gdrive_access_token.as_deref() else {
                        log(
                            LogLevel::Error,
                            "Google Drive native document export requires an OAuth access token",
                        );
                        failed = true;
                        break;
                    };
                    match rt.block_on(export_gdrive_file(
                        access_token,
                        &info.gdrive_root,
                        uri,
                        &content_type,
                    )) {
                        Ok((content, exported_content_type, exported_name)) => {
                            size = content.len();
                            content_type = exported_content_type;
                            name = exported_name;
                            content
                        }
                        Err(err) => {
                            log(LogLevel::Error, &err);
                            failed = true;
                            break;
                        }
                    }
                } else {
                    match rt.block_on(async { operator.read(uri).await }) {
                        Ok(content) => content.to_vec(),
                        Err(err) => {
                            log(LogLevel::Error, &err.to_string());
                            failed = true;
                            break;
                        }
                    }
                };

                // check file hash
                let file_hash = xxh3_64(&file_blob_content);
                if prev_file_hash == Some(file_hash) {
                    continue;
                }

                let file_id = Box::new(CString::new(uri).unwrap());
                let name = Box::new(CString::new(name).unwrap());
                let contet_type = Box::new(CString::new(content_type).unwrap());
                let metadata = Box::new(CString::new(metadata).unwrap());
                let dl_filename = rename_to_uuid(name.to_str().unwrap());
                let download_path = Path::new(dl_dir)
                    .join(dl_filename)
                    .to_string_lossy()
                    .to_string();
                let file_path = Box::new(CString::new(download_path.clone()).unwrap());

                if let Err(err) = fs::create_dir_all(dl_dir) {
                    log(
                        LogLevel::Error,
                        &format!("failed to create download directory '{dl_dir}': {err}"),
                    );
                    failed = true;
                    break;
                }

                if let Err(err) = fs::write(&download_path, file_blob_content) {
                    log(
                        LogLevel::Error,
                        &format!("failed to write downloaded file '{download_path}': {err}"),
                    );
                    failed = true;
                    break;
                };

                let added = Box::new(COutputFileInfo {
                    file_id: file_id.as_ptr(),
                    name: name.as_ptr(),
                    file_path: file_path.as_ptr(),
                    content_type: contet_type.as_ptr(),
                    size: size,
                    updated_at_utc: last_modified,
                    metadata: metadata.as_ptr(),
                    file_hash: file_hash,
                });

                nex_ref.chars.push(file_id);
                nex_ref.chars.push(name);
                nex_ref.chars.push(contet_type);
                nex_ref.chars.push(metadata);
                nex_ref.chars.push(file_path);
                nex_ref.result.data.added_or_updated = &*added;
                nex_ref.added_or_updated = Some(added);
                nex_ref.result.success = true;
                break;
            }
        }
        if nex_ref.added_or_updated.is_none() && !failed {
            if let Some(entry) = delete_entries.iter().next().cloned() {
                delete_entries.remove(&entry);
                let file_id = Box::new(CString::new(entry.clone()).unwrap());
                nex_ref.result.data.deleted = file_id.as_ptr();
                nex_ref.chars.push(file_id);
                nex_ref.result.success = true;
            }
        }
        if !failed && nex_ref.added_or_updated.is_none() && nex_ref.result.data.deleted.is_null() {
            nex_ref.result.success = true;
        }
    }

    let ret_ptr: *const CResult<CFileIterationInfo> = &*nex_ref.result;
    NEXT_REF.lock().unwrap().insert(ret_ptr as usize, nex_ref);
    return ret_ptr;
}

unsafe extern "C" fn free(value: *const c_void) {
    STR_REF.lock().unwrap().remove(&(value as usize));
    NEXT_REF.lock().unwrap().remove(&(value as usize));
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
