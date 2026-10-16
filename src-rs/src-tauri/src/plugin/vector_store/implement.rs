use crate::plugin::{
    common::{get_plugin, DisposableHnandler},
    manager::LibraryLoader,
    vector_store::interface::VectorStore,
};
use anydex_plugin_api_lib::{
    common::{Handler, PluginFn},
    interface::DynamicPluginHeader,
    vector_store::{interface::*, params::*, types::*},
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
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.schema);
            }
        }
    }

    fn get_handler(
        &self,
        param: &serde_json::Value,
        index_id: IndexId,
        db_dir: &str,
    ) -> Result<DisposableHnandler, String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let args = GetHandlerArgs {
                    params: param.clone(),
                    index_id: index_id,
                    db_dir: db_dir.to_string(),
                };
                let result = plugin_v1.apply_and_unwrap(args)?;

                let plugin_v1 = *plugin_v1;
                let free_handler = move |h| {
                    let args = FreeHandlerArgs { handler: h };
                    let result: Result<FreeHandlerData, String> = plugin_v1.apply_and_unwrap(args);
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
                    file_id: fild_id,
                };
                let _result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(());
            }
        }
    }

    fn clear_db(&self, handler: Handler) -> Result<(), String> {
        match self {
            DynamicVectorStore::V1(plugin_v1) => {
                let args = ClearDbArgs { handler };
                let _result = plugin_v1.apply_and_unwrap(args)?;
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
                let _result = plugin_v1.apply_and_unwrap(args)?;
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
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.results);
            }
        }
    }
}
