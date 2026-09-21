use param::{HandleInfo, Param};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CString};
use std::sync::{LazyLock, Mutex};

use anydex_plugin_api_lib::common::{resolve_dylib_name, to_string, CResult, CSlice, LogLevel};
use anydex_plugin_api_lib::interface::PluginInit;
use anydex_plugin_api_lib::vector_store::interface::{
    CChunk, CSearchHit, CSearchRequest, DynamicVectorStoreV1, FileId, Handler, IndexId,
    DYNAMIC_VECTOR_STORE_ABI_VERSION_V1,
};
use schemars::schema_for;

use crate::sqlite::sqlite_init;

pub mod param;
pub mod sqlite;

static PLUGIN: Mutex<Option<DynamicVectorStoreV1>> = Mutex::new(None);
static PLUGIN_INIT: Mutex<Option<PluginInit>> = Mutex::new(None);

fn log(level: LogLevel, msg: &str) {
    let plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
    if let Some(plugin_init) = *plugin_init_mtx {
        let cmsg = CString::new(msg).unwrap();
        unsafe { (plugin_init.log)(level, cmsg.as_ptr()) };
    }
}

pub unsafe extern "C" fn get_plugin(init: *const PluginInit) -> *const DynamicVectorStoreV1 {
    sqlite_init(&resolve_dylib_name("vector"));

    let mut plugin_mtx = PLUGIN.lock().unwrap();
    if (*plugin_mtx).is_none() {
        let mut plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
        *plugin_init_mtx = Some(unsafe { init.read().clone() });

        let new_plugin = DynamicVectorStoreV1 {
            abi_version: DYNAMIC_VECTOR_STORE_ABI_VERSION_V1,
            struct_size: std::mem::size_of::<DynamicVectorStoreV1>(),
            get_param_schema,
            get_handler,
            free_handler,
            delete_chunk,
            insert_chunk,
            clear_chunk,
            search,
            free,
        };
        *plugin_mtx = Some(new_plugin);
    }

    let ret = (*plugin_mtx).unwrap();
    return &raw const ret;
}

static STR_REF: LazyLock<Mutex<HashMap<usize, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLERS: LazyLock<Mutex<HashMap<Handler, HandleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLER_COUNTER: Mutex<u64> = Mutex::new(0);

unsafe extern "C" fn get_param_schema() -> *const c_char {
    let schema = schema_for!(Param);
    let schema_str = serde_json::to_string_pretty(&schema).unwrap();
    let ptr: *const u8 = schema_str.as_ptr();
    STR_REF.lock().unwrap().insert(ptr as usize, schema_str);

    return ptr as *const c_char;
}

unsafe extern "C" fn get_handler(params: *const c_char, index_id: IndexId) -> Handler {
    let schema_str = to_string(params);
    let schema = serde_json::from_str::<Param>(&schema_str).map_err(|err| err.to_string());
    if schema.is_ok() {
        let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
        let handler = (*handler_mtx).clone();
        (*handler_mtx) += 1;

        let info = HandleInfo {
            index_id,
            param: schema.unwrap(),
        };
        HANDLERS.lock().unwrap().insert(handler, info);

        handler
    } else {
        log(LogLevel::Error, &schema.unwrap_err());
        u64::MAX
    }
}

unsafe extern "C" fn free_handler(handler: Handler) {
    HANDLERS.lock().unwrap().remove(&handler);
}

unsafe extern "C" fn delete_chunk(handler: Handler, fild_id: FileId) {}

unsafe extern "C" fn clear_chunk(handler: Handler, index_id: IndexId) {}

unsafe extern "C" fn insert_chunk(handler: Handler, chunk: CChunk) {}

unsafe extern "C" fn search(
    handler: Handler,
    search_request: CSearchRequest,
) -> *const CResult<CSlice<CSearchHit>> {
}

unsafe extern "C" fn free(value: *const c_void) {
    STR_REF.lock().unwrap().remove(&(value as usize));
}
