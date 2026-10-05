use crate::plugin::{
    common::{get_plugin, DisposableHnandler},
    manager::LibraryLoader,
    vector_store::interface::VectorStore,
};
use anydex_plugin_api_lib::{
    common::{apply_and_unwrap, Handler},
    interface::DynamicPluginHeader,
    vector_store::{interface::*, params::*},
};
use libloading::Library;
use tracing::error;

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
                let args = GetParamSchemaArgs {};
                let result: GetParamSchemaData =
                    apply_and_unwrap(args, plugin_v1.get_param_schema, plugin_v1.free)?;
                return Ok(result.schema);
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
                let args = GetHandlerArgs {
                    params: param.clone(),
                    index_id: index_id,
                };
                let result: GetHandlerData =
                    apply_and_unwrap(args, plugin_v1.get_handler, plugin_v1.free)?;

                let free_handler_fn = plugin_v1.free_handler;
                let free_fn = plugin_v1.free;
                let free_handler = move |h| {
                    let args = FreeHandlerArgs { handler: h };
                    let result: Result<FreeHandlerData, String> =
                        apply_and_unwrap(args, free_handler_fn, free_fn);
                    if let Err(err) = result {
                        error!("{err}");
                    }
                };
                Ok(DisposableHnandler {
                    handler: result.handler,
                    free_fn: Box::new(free_handler),
                })
            }
        }
    }

    fn delete_chunks_file(&self, handler: Handler, fild_id: FileId) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let args = DeleteChunksArgs {
                    handler,
                    condition: DeleteCond::ByFileId(fild_id),
                };
                let _result: DeleteChunksData =
                    apply_and_unwrap(args, plugin_v1.delete_chunks, plugin_v1.free)?;
                return Ok(());
            }
        }
    }

    fn delete_chunks_indexer(&self, handler: Handler, indexer_id: IndexerId) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let args = DeleteChunksArgs {
                    handler,
                    condition: DeleteCond::ByIndexerId(indexer_id),
                };
                let _result: DeleteChunksData =
                    apply_and_unwrap(args, plugin_v1.delete_chunks, plugin_v1.free)?;
                return Ok(());
            }
        }
    }

    fn clear_chunks(&self, handler: Handler) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let args = ClearChunksArgs { handler };
                let _result: DeleteChunksData =
                    apply_and_unwrap(args, plugin_v1.clear_chunks, plugin_v1.free)?;
                return Ok(());
            }
        }
    }

    fn insert_chunks(&self, handler: Handler, chunks: &[Chunk]) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let args = InsertChunksArgs {
                    handler,
                    chunks: chunks.to_vec(),
                };
                let _result: InsertChunksData =
                    apply_and_unwrap(args, plugin_v1.insert_chunks, plugin_v1.free)?;
                return Ok(());
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
                let args = SearchArgs {
                    handler,
                    search_request: search_request.clone(),
                };
                let result: SearchData = apply_and_unwrap(args, plugin_v1.search, plugin_v1.free)?;
                return Ok(result.results);
            }
        }
    }
}
