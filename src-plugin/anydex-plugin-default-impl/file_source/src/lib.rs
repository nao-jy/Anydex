use std::{
    collections::{HashMap, VecDeque},
    ffi::{c_char, c_void, CString},
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
use chrono::Utc;
use opendal::{Entry, Operator};
use schemars::schema_for;
use tokio::runtime::Runtime;

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
    pub param: Param,
    pub last_indexed_files: HashMap<String, InputFileInfo>,
    pub download_dir: String,
    pub rt: Runtime,
    pub entries: VecDeque<Entry>,
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
            param: param_unwrapped,
            last_indexed_files: files_vec
                .iter()
                .enumerate()
                .map(|(_, value)| (value.file_id.clone(), value.clone()))
                .collect(),
            download_dir: to_string(download_dir),
            entries: lister.unwrap().into(),
            operator: operator,
            rt: rt,
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
    pub added_or_deleted: Option<Box<COutputFileInfo>>,
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
        added_or_deleted: None,
        chars: Vec::new(),
    };
    if let Some(info) = info_opt {
        let last_indexed_files = &info.last_indexed_files;

        while let Some(entry) = info.entries.pop_front() {
            let now = Utc::now().timestamp();
            let uri = entry.path();
            let last_modified = entry
                .metadata()
                .last_modified()
                .map_or(-1, |value| value.into_inner().as_second());
            let size = entry.metadata().content_length();
        }
        nex_ref.result.success = true;
    }

    let ret_ptr: *const CResult<CFileIterationInfo> = &*nex_ref.result;
    NEXT_REF.lock().unwrap().insert(ret_ptr as usize, nex_ref);
    return ret_ptr;
}

unsafe extern "C" fn free(value: *const c_void) {
    STR_REF.lock().unwrap().remove(&(value as usize));
    NEXT_REF.lock().unwrap().remove(&(value as usize));
}
