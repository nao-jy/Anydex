use crate::plugin::{
    common::{get_plugin, DisposableHnandler},
    manager::LibraryLoader,
    vector_store::interface::{Chunk, SearchHit, SearchRequest, VectorStore},
};
use anydex_plugin_api_lib::{
    common::{to_string, CSlice, Handler},
    interface::DynamicPluginHeader,
    vector_store::interface::{
        CChunk, CSearchHit, CSearchRequest, DynamicVectorStore, DynamicVectorStoreV1, FileId,
        IndexId, IndexerId, DYNAMIC_VECTOR_STORE_ABI_VERSION_V1,
    },
};
use chrono::{DateTime, Utc};
use libloading::Library;
use std::ffi::{c_void, CString};

impl LibraryLoader for DynamicVectorStore {
    fn load_library(library: &Library) -> Result<Self, String> {
        let plugin_header_ptr: *const DynamicPluginHeader = get_plugin(library)?;
        let plugin_header = unsafe { std::ptr::read(plugin_header_ptr) };

        match plugin_header.abi_version {
            DYNAMIC_VECTOR_STORE_ABI_VERSION_V1 => {
                let required_size = std::mem::size_of::<DynamicVectorStoreV1>();
                if plugin_header.struct_size != required_size {
                    return Err(format!(
                        "embedder API table is not same as expected: got {}, need {}",
                        plugin_header.struct_size, required_size
                    ));
                }
                let plugin_api =
                    unsafe { std::ptr::read(plugin_header_ptr.cast::<DynamicVectorStoreV1>()) };
                Ok(DynamicVectorStore::V1(plugin_api))
            }
            version => Err(format!("unsupported version: {version}")),
        }
    }
}

impl VectorStore for DynamicVectorStore {
    fn get_param_schema(&self) -> Result<serde_json::Value, String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let raw_schema = unsafe { (plugin_v1.get_param_schema)() };
                if raw_schema.is_null() {
                    return Err("get_param_schema returns null".into());
                }

                let schema = to_string(raw_schema);
                unsafe { (plugin_v1.free)(raw_schema as *mut c_void) };
                serde_json::from_str(&schema).map_err(|err| err.to_string())
            }
        }
    }

    fn get_handler(
        &self,
        param: &serde_json::Value,
        index_id: IndexId,
    ) -> Result<DisposableHnandler, String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let params =
                    CString::new(serde_json::to_string(param).map_err(|err| err.to_string())?)
                        .map_err(|err| err.to_string())?;
                let handler = unsafe { (plugin_v1.get_handler)(params.as_ptr(), index_id) };
                let free_handler = plugin_v1.free_handler;
                Ok(DisposableHnandler {
                    handler,
                    free_fn: Box::new(move |h| unsafe { free_handler(h) }),
                })
            }
        }
    }

    fn delete_chunk_file(&self, handler: Handler, fild_id: FileId) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                Ok(unsafe { (plugin_v1.delete_chunk_file)(handler, fild_id) })
            }
        }
    }

    fn delete_chunk_indexer(&self, handler: Handler, indexer_id: IndexerId) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                Ok(unsafe { (plugin_v1.delete_chunk_indexer)(handler, indexer_id) })
            }
        }
    }

    fn clear_chunk(&self, handler: Handler) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => Ok(unsafe { (plugin_v1.clear_chunk)(handler) }),
        }
    }

    fn insert_chunk(&self, handler: Handler, chunk: &Chunk) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let curi = CString::new(chunk.uri.as_str()).map_err(|err| err.to_string())?;
                let ccontent_type =
                    CString::new(chunk.content_type.as_str()).map_err(|err| err.to_string())?;
                let ccontent = match &chunk.content {
                    Some(content) => {
                        let ccontent =
                            CString::new(content.as_str()).map_err(|err| err.to_string())?;
                        Some(ccontent)
                    }
                    None => None,
                };
                let ccontent_vector = match &chunk.content_vector {
                    Some(content_vec) => Some(CSlice {
                        ptr: content_vec.as_ptr(),
                        length: content_vec.len(),
                    }),
                    None => None,
                };
                let metadata_str =
                    CString::new(chunk.metadata.to_string()).map_err(|err| err.to_string())?;
                let cchunk = CChunk {
                    id: chunk.id,
                    index_id: chunk.index_id,
                    indexer_id: chunk.indexer_id,
                    file_id: chunk.file_id,
                    uri: curi.as_ptr(),
                    content_type: ccontent_type.as_ptr(),
                    content: ccontent
                        .as_ref()
                        .map_or(std::ptr::null(), |content| content.as_ptr()),
                    content_vector: ccontent_vector
                        .as_ref()
                        .map_or(std::ptr::null(), |cvec| cvec),
                    metadata: metadata_str.as_ptr(),
                    created_at_utc: chunk.created_at.timestamp(),
                    updated_at_utc: chunk.updated_at.timestamp(),
                };
                Ok(unsafe { (plugin_v1.insert_chunk)(handler, cchunk) })
            }
        }
    }
    fn search(
        &self,
        handler: Handler,
        search_request: &SearchRequest,
    ) -> Result<Vec<SearchHit>, String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let ccond = CString::new(search_request.additional_cond_sql.clone())
                    .map_err(|err| err.to_string())?;
                let csearch = CSearchRequest {
                    embedding: CSlice {
                        ptr: search_request.embedding.as_ptr(),
                        length: search_request.embedding.len(),
                    },
                    additional_cond_sql: ccond.as_ptr(),
                    limit: search_request.limit,
                    offset: search_request.offset,
                };
                let result_ptr = unsafe { (plugin_v1.search)(handler, csearch) };
                let result = unsafe { result_ptr.read() };
                if !result.success {
                    return Err("search failed".into());
                }
                let result_vec: Vec<CSearchHit> = result.data.into();
                let mut ret = Vec::with_capacity(result_vec.len());
                for res in result_vec {
                    let content = if res.chunk.content.is_null() {
                        None
                    } else {
                        Some(to_string(res.chunk.content))
                    };
                    let content_vector = if res.chunk.content_vector.is_null() {
                        None
                    } else {
                        let v = unsafe { res.chunk.content_vector.read() };
                        Some(v.into())
                    };
                    let metadata = serde_json::Value::from(to_string(res.chunk.metadata));
                    let now = Utc::now();
                    let chunk = Chunk {
                        id: res.chunk.id,
                        index_id: res.chunk.index_id,
                        indexer_id: res.chunk.indexer_id,
                        file_id: res.chunk.file_id,
                        uri: to_string(res.chunk.uri),
                        content_type: to_string(res.chunk.content_type),
                        content: content,
                        content_vector: content_vector,
                        metadata: metadata,
                        created_at: DateTime::from_timestamp(res.chunk.created_at_utc, 0)
                            .unwrap_or(now),
                        updated_at: DateTime::from_timestamp(res.chunk.updated_at_utc, 0)
                            .unwrap_or(now),
                    };
                    let search = SearchHit {
                        chunk: chunk,
                        score: res.score,
                    };
                    ret.push(search);
                }
                unsafe { (plugin_v1.free)(result_ptr as *mut c_void) };
                Ok(ret)
            }
        }
    }
}
