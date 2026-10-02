use anydex_plugin_api_lib::{
    common::{to_string, CSlice},
    file_source::interface::{
        CInputFileInfo, DynamicFileSource, DynamicFileSourceV1, Handler,
        DYNAMIC_FILE_SOURCE_ABI_VERSION_V1,
    },
    interface::DynamicPluginHeader,
};
use chrono::{DateTime, Utc};
use libloading::Library;
use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString},
};

use crate::plugin::{
    common::get_plugin,
    file_source::interface::{FileIterationInfo, FileSource, InputFileInfo, OutputFileInfo},
    manager::LibraryLoader,
};

impl LibraryLoader for DynamicFileSource {
    fn load_library(library: &Library) -> Result<Self, String> {
        let plugin_header_ptr: *const DynamicPluginHeader = get_plugin(library)?;
        let plugin_header = unsafe { std::ptr::read(plugin_header_ptr) };

        match plugin_header.abi_version {
            DYNAMIC_FILE_SOURCE_ABI_VERSION_V1 => {
                let required_size = std::mem::size_of::<DynamicFileSourceV1>();
                if plugin_header.struct_size != required_size {
                    return Err(format!(
                        "embedder API table is not same as expected: got {}, need {}",
                        plugin_header.struct_size, required_size
                    ));
                }
                let plugin_api =
                    unsafe { std::ptr::read(plugin_header_ptr.cast::<DynamicFileSourceV1>()) };
                Ok(DynamicFileSource::V1(plugin_api))
            }
            version => Err(format!("unsupported version: {version}")),
        }
    }
}

fn object_to_hash_map(value: serde_json::Value) -> HashMap<String, String> {
    match value {
        serde_json::Value::Object(map) => map
            .into_iter()
            .filter_map(|(key, value)| value.as_str().map(|value| (key, value.to_owned())))
            .collect(),
        _ => HashMap::new(),
    }
}

impl FileSource for DynamicFileSource {
    fn get_param_schema(&self) -> Result<serde_json::Value, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let raw_schema = unsafe { (plugin_v1.get_param_schema)() };
                if raw_schema.is_null() {
                    return Err("get_param_schema returns null".into());
                }

                let schema = unsafe { CStr::from_ptr(raw_schema).to_string_lossy().into_owned() };
                unsafe { (plugin_v1.free)(raw_schema as *mut c_void) };
                serde_json::from_str(&schema).map_err(|err| err.to_string())
            }
        }
    }

    fn get_handler(
        &self,
        param: &serde_json::Value,
        last_indexed_files: &[InputFileInfo],
        access_token: Option<String>,
        download_dir: &str,
    ) -> Result<Handler, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let params =
                    CString::new(serde_json::to_string(param).map_err(|err| err.to_string())?)
                        .map_err(|err| err.to_string())?;
                let download_dir = CString::new(download_dir).map_err(|err| err.to_string())?;
                let access_token = match access_token {
                    Some(token) => Some(CString::new(token).map_err(|err| err.to_string())?),
                    None => None,
                };

                let mut cstrings = Vec::with_capacity(last_indexed_files.len());
                let mut cfiles = Vec::with_capacity(last_indexed_files.len());

                for info in last_indexed_files {
                    let fid = CString::new(info.file_id.clone()).map_err(|err| err.to_string())?;

                    cfiles.push(CInputFileInfo {
                        file_id: fid.as_ptr(),
                        size: info.size,
                        updated_at_utc: info.updated_at.timestamp(),
                        file_hash: info.file_hash,
                    });

                    cstrings.push(fid);
                }
                let handler = unsafe {
                    (plugin_v1.get_handler)(
                        params.as_ptr(),
                        CSlice {
                            ptr: cfiles.as_ptr(),
                            length: cfiles.len(),
                        },
                        access_token.map(|t| t.as_ptr()).unwrap_or(std::ptr::null()),
                        download_dir.as_ptr(),
                    )
                };
                Ok(handler)
            }
        }
    }

    fn free_handler(&self, handler: Handler) -> Result<(), String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => Ok(unsafe { (plugin_v1.free_handler)(handler) }),
        }
    }

    fn next(&self, handler: Handler) -> Result<Option<FileIterationInfo>, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let result_ptr = unsafe { (plugin_v1.next)(handler) };
                let result = unsafe { result_ptr.read() };
                let mut ret: Result<Option<FileIterationInfo>, String> = Ok(None);
                if !result.success {
                    ret = Err("next_added_or_changed failed".into());
                }
                if result.data.added_or_updated.is_null() && result.data.deleted.is_null() {
                    ret = Ok(None);
                }
                // added or updated
                if !result.data.added_or_updated.is_null() {
                    let cfile = unsafe { result.data.added_or_updated.read() };
                    let metadata = serde_json::Value::from(to_string(cfile.metadata));

                    let added_or_updated = OutputFileInfo {
                        file_id: to_string(cfile.file_id),
                        name: to_string(cfile.name),
                        file_path: to_string(cfile.file_path),
                        content_type: to_string(cfile.content_type),
                        size: cfile.size,
                        updated_at: DateTime::from_timestamp(cfile.updated_at_utc, 0)
                            .unwrap_or(Utc::now()),
                        metadata: object_to_hash_map(metadata),
                        file_hash: cfile.file_hash,
                    };
                    ret = Ok(Some(FileIterationInfo::AddedOrDeleted(added_or_updated)));
                }
                // deleted
                if !result.data.deleted.is_null() {
                    let deleted: String = to_string(result.data.deleted);
                    ret = Ok(Some(FileIterationInfo::Deleted(deleted)));
                }
                unsafe { (plugin_v1.free)(result_ptr as *mut c_void) };
                ret
            }
        }
    }
}
