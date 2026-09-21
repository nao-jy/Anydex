use anydex_plugin_api_lib::{
    common::{to_string, CPair, CSlice},
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
    file_source::interface::{FileId, FileSource, InputFileInfo, OutputFileInfo},
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
        download_dir: &str,
    ) -> Result<Handler, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let params =
                    CString::new(serde_json::to_string(param).map_err(|err| err.to_string())?)
                        .map_err(|err| err.to_string())?;
                let download_dir = CString::new(download_dir).map_err(|err| err.to_string())?;

                let mut cstrings = Vec::with_capacity(last_indexed_files.len());
                let mut cfiles = Vec::with_capacity(last_indexed_files.len());

                for info in last_indexed_files {
                    let fid = CString::new(info.file_id.clone()).map_err(|err| err.to_string())?;

                    cfiles.push(CInputFileInfo {
                        file_id: fid.as_ptr(),
                        size: info.size,
                        updated_at_utc: info.updated_at.timestamp(),
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

    fn next_added_or_changed(&self, handler: Handler) -> Result<Option<OutputFileInfo>, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let result_ptr = unsafe { (plugin_v1.next_added_or_changed)(handler) };
                let result = unsafe { result_ptr.read() };
                if !result.success {
                    return Err("next_added_or_changed failed".into());
                } else if result.data.is_null() {
                    return Ok(None);
                }

                let cfile = unsafe { result.data.read() };
                let mut metadata = HashMap::new();
                let cmetadata_vec: Vec<CPair<*const i8, *const i8>> = cfile.metadata.into();
                for data in cmetadata_vec {
                    metadata.insert(to_string(data.key), to_string(data.value));
                }

                let ret = OutputFileInfo {
                    file_id: unsafe {
                        CStr::from_ptr(cfile.file_id).to_string_lossy().into_owned()
                    },
                    name: unsafe { CStr::from_ptr(cfile.name).to_string_lossy().into_owned() },
                    file_path: unsafe {
                        CStr::from_ptr(cfile.file_path)
                            .to_string_lossy()
                            .into_owned()
                    },
                    content_type: unsafe {
                        CStr::from_ptr(cfile.content_type)
                            .to_string_lossy()
                            .into_owned()
                    },
                    size: cfile.size,
                    updated_at: DateTime::from_timestamp(cfile.updated_at_utc, 0)
                        .unwrap_or(Utc::now()),
                    metadata: metadata,
                };
                unsafe { (plugin_v1.free)(result_ptr as *mut c_void) };
                Ok(Some(ret))
            }
        }
    }

    fn next_deleted(&self, handler: Handler) -> Result<Option<FileId>, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let result_ptr = unsafe { (plugin_v1.next_deleted)(handler) };
                let result = unsafe { result_ptr.read() };
                if !result.success {
                    return Err("next_added_or_changed failed".into());
                } else if result.data.is_null() {
                    return Ok(None);
                }
                let ret = to_string(result.data);
                unsafe { (plugin_v1.free)(result_ptr as *mut c_void) };
                Ok(Some(ret))
            }
        }
    }
}
