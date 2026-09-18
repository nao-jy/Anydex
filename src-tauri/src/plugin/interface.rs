use libloading::{Library, Symbol};

// struct header part
// plugin must start with this header
#[repr(C)]
pub struct DynamicPluginHeader {
    pub abi_version: i32,
    pub struct_size: usize,
}

// Plugin entrypoint: `get_plugin`.
pub type GetPlugin<T> = unsafe extern "C" fn() -> *const T;

// Get plugin pointer
pub fn get_plugin<T>(library: &Library) -> Result<*const T, String> {
    let get_plugin: Symbol<GetPlugin<T>> = unsafe {
        library
            .get(b"get_plugin")
            .map_err(|error| format!("missing get_plugin: {error}"))?
    };
    let plugin = unsafe { get_plugin() };
    if plugin.is_null() {
        return Err("get_plugin returned a null pointer".to_string());
    }
    Ok(plugin)
}
