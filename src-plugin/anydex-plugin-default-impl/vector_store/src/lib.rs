use std::ffi::{c_char, c_void, CString};
use std::sync::Mutex;

use anydex_plugin_api_lib::common::{CResult, CSlice, LogLevel};
use anydex_plugin_api_lib::interface::PluginInit;
use anydex_plugin_api_lib::vector_store::interface::{
    CChunk, CSearchHit, CSearchRequest, DynamicVectorStoreV1, FileId, Handler, IndexId,
    DYNAMIC_VECTOR_STORE_ABI_VERSION_V1,
};
use schemars::schema_for;

pub mod param;

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
            search,
            free,
        };
        *plugin_mtx = Some(new_plugin);
    }

    let ret = (*plugin_mtx).unwrap();
    return &raw const ret;
}

unsafe extern "C" fn get_param_schema() -> *const c_char {
    let schema = schema_for!(param::Param);
    let ret = serde_json::to_string_pretty(&schema).unwrap();
}

unsafe extern "C" fn get_handler(params: *const c_char, index_id: IndexId) -> Handler {}

unsafe extern "C" fn free_handler(handler: Handler) {}

unsafe extern "C" fn delete_chunk(handler: Handler, fild_id: FileId) {}

unsafe extern "C" fn insert_chunk(handler: Handler, chunk: CChunk) {}

unsafe extern "C" fn search(
    handler: Handler,
    search_request: CSearchRequest,
) -> *const CResult<CSlice<CSearchHit>> {
}

unsafe extern "C" fn free(value: *const c_void) {}
