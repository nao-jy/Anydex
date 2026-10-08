use crate::{
    common::{APIResultSerialized, PluginFn},
    embedder::params::*,
};
use std::ffi::c_char;

/// ABI version implemented by this host.
pub const DYNAMIC_EMBEDDER_ABI_VERSION_V1: i32 = 1;

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DynamicEmbedderV1 {
    /// Must equal [`DYNAMIC_EMBEDDER_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema:
        unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub get_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub free_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub embed: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,

    pub free: unsafe extern "C" fn(value: APIResultSerialized),
}

impl PluginFn<GetParamSchemaArgs, GetParamSchemaData> for DynamicEmbedderV1 {
    fn apply_and_unwrap(&self, args: GetParamSchemaArgs) -> Result<GetParamSchemaData, String> {
        super::super::common::apply_and_unwrap(args, self.get_param_schema, self.free)
    }
}

impl PluginFn<GetHandlerArgs, GetHandlerData> for DynamicEmbedderV1 {
    fn apply_and_unwrap(&self, args: GetHandlerArgs) -> Result<GetHandlerData, String> {
        super::super::common::apply_and_unwrap(args, self.get_handler, self.free)
    }
}

impl PluginFn<FreeHandlerArgs, FreeHandlerData> for DynamicEmbedderV1 {
    fn apply_and_unwrap(&self, args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
        super::super::common::apply_and_unwrap(args, self.free_handler, self.free)
    }
}

impl PluginFn<EmbedArgs, EmbedData> for DynamicEmbedderV1 {
    fn apply_and_unwrap(&self, args: EmbedArgs) -> Result<EmbedData, String> {
        super::super::common::apply_and_unwrap(args, self.embed, self.free)
    }
}

#[derive(Clone)]
pub enum DynamicEmbedder {
    V1(DynamicEmbedderV1),
}
