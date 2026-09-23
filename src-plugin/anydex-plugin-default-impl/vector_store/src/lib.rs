use param::Param;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CString};
use std::sync::{LazyLock, Mutex};

use anydex_plugin_api_lib::common::{resolve_dylib_name, to_string, CResult, CSlice, LogLevel};
use anydex_plugin_api_lib::interface::PluginInit;
use anydex_plugin_api_lib::vector_store::interface::{
    CChunk, CSearchHit, CSearchRequest, DynamicVectorStoreV1, FileId, Handler, IndexId,
    DYNAMIC_VECTOR_STORE_ABI_VERSION_V1, INVALID_HANDLE,
};
use schemars::schema_for;

use crate::sqlite::Chunk;

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_plugin(init: *const PluginInit) -> *const DynamicVectorStoreV1 {
    let init_result = sqlite::init_sqlite(&resolve_dylib_name("vector"));
    if let Err(err) = init_result {
        log(LogLevel::Error, &err);
        return std::ptr::null();
    }

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

    let ret = plugin_mtx.as_ref().unwrap();
    return &raw const *ret;
}

struct HandleInfo {
    pub param: Param,
    pub index_id: IndexId,
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
        sqlite::init_table(index_id, &info.param).unwrap_or_else(|err| log(LogLevel::Error, &err));
        HANDLERS.lock().unwrap().insert(handler, info);
        handler
    } else {
        log(LogLevel::Error, &schema.unwrap_err());
        INVALID_HANDLE
    }
}

unsafe extern "C" fn free_handler(handler: Handler) {
    HANDLERS.lock().unwrap().remove(&handler);
}

unsafe extern "C" fn delete_chunk(handler: Handler, file_id: FileId) {
    let info_mtx = HANDLERS.lock().unwrap();
    let info_opt = (*info_mtx).get(&handler);
    if let Some(info) = info_opt {
        let _ = sqlite::delete(info.index_id, file_id).map_err(|err| log(LogLevel::Error, &err));
    }
}

unsafe extern "C" fn clear_chunk(handler: Handler) {
    let info_mtx = HANDLERS.lock().unwrap();
    let info_opt = (*info_mtx).get(&handler);
    if let Some(info) = info_opt {
        sqlite::clear(info.index_id).unwrap_or_else(|err| log(LogLevel::Error, &err));
    }
}

unsafe extern "C" fn insert_chunk(handler: Handler, cchunk: CChunk) {
    let info_mtx = HANDLERS.lock().unwrap();
    let info_opt = (*info_mtx).get(&handler);
    if let Some(info) = info_opt {
        let chunk = Chunk {
            id: cchunk.id,
            index_id: cchunk.index_id,
            indexer_id: cchunk.indexer_id,
            file_id: cchunk.file_id,
            uri: to_string(cchunk.uri),
            content_type: to_string(cchunk.content_type),
            content: if cchunk.content.is_null() {
                None
            } else {
                Some(to_string(cchunk.content))
            },
            content_vector: if cchunk.content_vector.is_null() {
                None
            } else {
                let cslice = unsafe { cchunk.content_vector.read() };
                Some(cslice.into())
            },
            metadata: serde_json::Value::from(to_string(cchunk.metadata)),
            created_at_utc: cchunk.created_at_utc,
            updated_at_utc: cchunk.updated_at_utc,
        };
        sqlite::insert(info.index_id, chunk).unwrap_or_else(|err| log(LogLevel::Error, &err));
    }
}

struct SearchResultRef {
    pub str_ref: Vec<Box<CString>>,
    pub search_hits: Vec<CSearchHit>,
    pub result: CResult<CSlice<CSearchHit>>,
}

// These FFI-backed values carry raw pointers and therefore are not automatically
// `Send`/`Sync` across threads. The shared static is intentionally used as a
// process-global registry, so the pointer-bearing payloads must be treated as
// thread-safe by the FFI boundary contract.
unsafe impl Send for SearchResultRef {}
unsafe impl Sync for SearchResultRef {}

static SEARCH_RESULT_REF: LazyLock<Mutex<HashMap<usize, Box<SearchResultRef>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

unsafe extern "C" fn search(
    handler: Handler,
    search_request: CSearchRequest,
) -> *const CResult<CSlice<CSearchHit>> {
    let info_mtx = HANDLERS.lock().unwrap();
    let info_opt = (*info_mtx).get(&handler);

    let mut ret_info = Box::new(SearchResultRef {
        str_ref: Vec::new(),
        search_hits: Vec::new(),
        result: CResult {
            success: false,
            data: CSlice {
                ptr: std::ptr::null(),
                length: 0,
            },
        },
    });
    if let Some(info) = info_opt {
        let vec: Vec<f32> = search_request.embedding.into();
        let cond = to_string(search_request.additional_cond_sql);
        let result_res = sqlite::search(
            info.index_id,
            &vec,
            &cond,
            search_request.offset,
            search_request.limit,
        );
        if let Ok(result) = result_res {
            ret_info.search_hits.reserve(result.len());
            for (chunk, score) in result {
                let uri: Box<CString> = Box::from(CString::new(chunk.uri).unwrap());
                let content_type: Box<CString> =
                    Box::from(CString::new(chunk.content_type).unwrap());
                let content: Option<Box<CString>> = if let Some(c) = chunk.content {
                    Some(Box::from(CString::new(c).unwrap()))
                } else {
                    None
                };
                let metadata: Box<CString> =
                    Box::new(CString::new(chunk.metadata.to_string()).unwrap());
                let cchunk = CChunk {
                    id: chunk.id,
                    index_id: chunk.index_id,
                    indexer_id: chunk.indexer_id,
                    file_id: chunk.file_id,
                    uri: uri.as_ptr(),
                    content_type: content_type.as_ptr(),
                    content: if let Some(c) = &content {
                        c.as_ptr()
                    } else {
                        std::ptr::null()
                    },
                    content_vector: std::ptr::null(),
                    metadata: metadata.as_ptr(),
                    created_at_utc: chunk.created_at_utc,
                    updated_at_utc: chunk.updated_at_utc,
                };
                let search_hit = CSearchHit {
                    chunk: cchunk,
                    score,
                };
                ret_info.str_ref.push(uri);
                ret_info.str_ref.push(content_type);
                ret_info.str_ref.push(metadata);
                if let Some(c) = content {
                    ret_info.str_ref.push(c);
                }
                ret_info.search_hits.push(search_hit);
            }
            ret_info.result.data.ptr = ret_info.search_hits.as_ptr();
            ret_info.result.data.length = ret_info.search_hits.len();
            ret_info.result.success = true;
        } else if let Err(err) = result_res {
            log(LogLevel::Error, &err)
        }
    }
    let ret_ptr = &raw const ret_info.result;
    SEARCH_RESULT_REF
        .lock()
        .unwrap()
        .insert(ret_ptr as usize, ret_info);
    return ret_ptr;
}

unsafe extern "C" fn free(value: *const c_void) {
    STR_REF.lock().unwrap().remove(&(value as usize));
    SEARCH_RESULT_REF.lock().unwrap().remove(&(value as usize));
}
