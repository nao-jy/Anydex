use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::{c_char, c_void, CString},
    fs,
    path::Path,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{to_string, CResult, CSlice, LogLevel},
    file_source::interface::{
        CFileIterationInfo, CInputFileInfo, COutputFileInfo, DynamicFileSourceV1, Handler,
        DYNAMIC_FILE_SOURCE_ABI_VERSION_V1, INVALID_HANDLE,
    },
    interface::PluginInit,
};
use opendal::{Entry, Operator};
use schemars::schema_for;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::param::Param;

pub mod param;

static PLUGIN: Mutex<Option<DynamicFileSourceV1>> = Mutex::new(None);
static PLUGIN_INIT: Mutex<Option<PluginInit>> = Mutex::new(None);

fn log(level: LogLevel, msg: &str) {
    let plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
    if let Some(plugin_init) = *plugin_init_mtx {
        let cmsg = CString::new(msg).unwrap();
        unsafe { (plugin_init.log)(level, cmsg.as_ptr()) };
    }
}

pub unsafe extern "C" fn get_plugin(init: *const PluginInit) -> *const DynamicFileSourceV1 {
    let mut plugin_mtx = PLUGIN.lock().unwrap();
    if (*plugin_mtx).is_none() {
        let mut plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
        *plugin_init_mtx = Some(unsafe { init.read().clone() });

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
}
struct HandleInfo {
    pub last_indexed_files: HashMap<String, InputFileInfo>,
    pub download_dir: String,
    pub rt: Runtime,
    pub entries: VecDeque<Entry>,
    pub delete_entries: HashSet<String>,
    pub operator: Operator,
}

static STR_REF: LazyLock<Mutex<HashMap<usize, Box<CString>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLERS: LazyLock<Mutex<HashMap<Handler, HandleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLER_COUNTER: Mutex<u64> = Mutex::new(0);

unsafe extern "C" fn get_param_schema() -> *const c_char {
    let schema = schema_for!(Param);
    let schema_str: Box<CString> =
        Box::from(CString::new(serde_json::to_string_pretty(&schema).unwrap()).unwrap());
    let ptr = schema_str.as_ptr();
    STR_REF.lock().unwrap().insert(ptr as usize, schema_str);

    return ptr;
}

fn object_to_list(value: serde_json::Value) -> Vec<(String, String)> {
    match value {
        serde_json::Value::Object(map) => map
            .into_iter()
            .filter_map(|(key, value)| value.as_str().map(|value| (key, value.to_owned())))
            .collect(),
        _ => Vec::new(),
    }
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
    download_dir: *const c_char,
) -> Handler {
    let param_str = to_string(params);
    let param_result = serde_json::from_str::<Param>(&param_str).map_err(|err| err.to_string());
    if param_result.is_ok() {
        let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
        let handler = (*handler_mtx).clone();
        (*handler_mtx) += 1;

        let cfiles_vec: Vec<CInputFileInfo> = last_indexed_files.into();
        let files_vec: Vec<InputFileInfo> = cfiles_vec
            .iter()
            .map(|f| InputFileInfo {
                file_id: to_string(f.file_id),
                size: f.size,
                updated_at_utc: f.updated_at_utc,
            })
            .collect();

        let param_unwrapped = param_result.unwrap();

        let operator_res = Operator::via_iter(
            param_unwrapped.service_name.clone(),
            object_to_list(serde_json::from_str(&param_str).unwrap()),
        );
        if operator_res.is_err() {
            log(LogLevel::Error, &operator_res.unwrap_err().to_string());
            return INVALID_HANDLE;
        }
        let operator = operator_res.unwrap();
        let rt = Runtime::new().unwrap();
        let lister = rt.block_on(async { operator.list_with("").recursive(true).await });
        if lister.is_err() {
            log(LogLevel::Error, &lister.unwrap_err().to_string());
            return INVALID_HANDLE;
        }

        let info = HandleInfo {
            last_indexed_files: files_vec
                .iter()
                .enumerate()
                .map(|(_, value)| (value.file_id.clone(), value.clone()))
                .collect(),
            download_dir: to_string(download_dir),
            entries: lister.unwrap().into(),
            operator: operator,
            rt: rt,
            delete_entries: files_vec.iter().map(|v| v.file_id.clone()).collect(),
        };

        HANDLERS.lock().unwrap().insert(handler, info);
        handler
    } else {
        log(LogLevel::Error, &param_result.unwrap_err());
        INVALID_HANDLE
    }
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

        if info.entries.is_empty() || delete_entries.is_empty() {
            nex_ref.result.success = true;
        }
        while let Some(entry) = info.entries.pop_front() {
            let uri = entry.path();
            let last_modified = entry
                .metadata()
                .last_modified()
                .map_or(-1, |value| value.into_inner().as_second());
            let size = entry.metadata().content_length() as usize;
            let name = entry.name();
            let content_type = entry.metadata().content_type().map_or(
                mime_guess::from_path(&name)
                    .first_or_octet_stream()
                    .to_string(),
                |f| f.to_string(),
            );
            let metadata = serde_json::Value::Null.to_string();
            let dl_filename = rename_to_uuid(name);
            let download_path = Path::new(dl_dir)
                .join(dl_filename)
                .to_string_lossy()
                .to_string();

            delete_entries.remove(uri);

            let prev_entry = last_indexed_files.get(uri);

            // added
            let added_or_updated = if prev_entry.is_none() {
                true
            } else {
                let prev_entry_unwrapped = prev_entry.unwrap();
                prev_entry_unwrapped.size != size
                    || (prev_entry_unwrapped.updated_at_utc < last_modified || last_modified < 0)
            };
            if added_or_updated {
                let file_id = Box::new(CString::new(uri).unwrap());
                let name = Box::new(CString::new(name).unwrap());
                let contet_type = Box::new(CString::new(content_type).unwrap());
                let metadata = Box::new(CString::new(metadata).unwrap());
                let file_path = Box::new(CString::new(download_path.clone()).unwrap());

                let added = Box::new(COutputFileInfo {
                    file_id: file_id.as_ptr(),
                    name: name.as_ptr(),
                    file_path: file_path.as_ptr(),
                    content_type: contet_type.as_ptr(),
                    size: size,
                    updated_at_utc: last_modified,
                    metadata: metadata.as_ptr(),
                });

                let file_blob_res = rt.block_on(async { operator.read(&uri).await });
                if file_blob_res.is_err() {
                    log(LogLevel::Debug, &file_blob_res.unwrap_err().to_string());
                    break;
                }
                let fs_result = fs::write(&download_path, file_blob_res.unwrap().to_vec());
                if fs_result.is_err() {
                    log(LogLevel::Debug, &fs_result.unwrap_err().to_string());
                    break;
                }

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
        if nex_ref.added_or_updated.is_none() {
            while let Some(entry) = info.delete_entries.iter().next() {
                let file_id = Box::new(CString::new(entry.clone()).unwrap());
                nex_ref.result.data.deleted = file_id.as_ptr();
                nex_ref.chars.push(file_id);
                nex_ref.result.success = true;
            }
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
