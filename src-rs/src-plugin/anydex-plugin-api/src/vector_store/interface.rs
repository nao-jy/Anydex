use crate::{
    common::{APIResultSerialized, PluginFn},
    vector_store::params::*,
};
use std::ffi::c_char;

/// ABI version implemented by this host.
pub const DYNAMIC_VECTOR_STORE_ABI_VERSION_V1: i32 = 1;

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DynamicVectorStoreV1 {
    /// Must equal [`DYNAMIC_VECTOR_STORE_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema:
        unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub get_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub free_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,

    pub delete_chunks: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub insert_chunks: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub clear_chunks: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub search: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,

    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: APIResultSerialized),
}

impl PluginFn<GetParamSchemaArgs, GetParamSchemaData> for DynamicVectorStoreV1 {
    fn apply_and_unwrap(&self, args: GetParamSchemaArgs) -> Result<GetParamSchemaData, String> {
        super::super::common::apply_and_unwrap(args, self.get_param_schema, self.free)
    }
}

impl PluginFn<GetHandlerArgs, GetHandlerData> for DynamicVectorStoreV1 {
    fn apply_and_unwrap(&self, args: GetHandlerArgs) -> Result<GetHandlerData, String> {
        super::super::common::apply_and_unwrap(args, self.get_handler, self.free)
    }
}

impl PluginFn<FreeHandlerArgs, FreeHandlerData> for DynamicVectorStoreV1 {
    fn apply_and_unwrap(&self, args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
        super::super::common::apply_and_unwrap(args, self.free_handler, self.free)
    }
}

impl PluginFn<DeleteChunksArgs, DeleteChunksData> for DynamicVectorStoreV1 {
    fn apply_and_unwrap(&self, args: DeleteChunksArgs) -> Result<DeleteChunksData, String> {
        super::super::common::apply_and_unwrap(args, self.delete_chunks, self.free)
    }
}

impl PluginFn<InsertChunksArgs, InsertChunksData> for DynamicVectorStoreV1 {
    fn apply_and_unwrap(&self, args: InsertChunksArgs) -> Result<InsertChunksData, String> {
        super::super::common::apply_and_unwrap(args, self.insert_chunks, self.free)
    }
}

impl PluginFn<ClearChunksArgs, ClearChunksData> for DynamicVectorStoreV1 {
    fn apply_and_unwrap(&self, args: ClearChunksArgs) -> Result<ClearChunksData, String> {
        super::super::common::apply_and_unwrap(args, self.clear_chunks, self.free)
    }
}

impl PluginFn<SearchArgs, SearchData> for DynamicVectorStoreV1 {
    fn apply_and_unwrap(&self, args: SearchArgs) -> Result<SearchData, String> {
        super::super::common::apply_and_unwrap(args, self.search, self.free)
    }
}

#[derive(Clone)]
pub enum DynamicVectorStore {
    V1(DynamicVectorStoreV1),
}
