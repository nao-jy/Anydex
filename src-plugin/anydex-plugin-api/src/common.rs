use std::env::consts::{ARCH, DLL_SUFFIX, OS};
use std::ffi::{c_char, CStr, CString};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum APIResult<T> {
    Ok { data: T },
    Err { error: String },
}

pub type APIResultSerialized = *const c_char;

pub fn result_to_api_result<T: Clone>(result: Result<T, String>) -> APIResult<T> {
    match result {
        Ok(data) => APIResult::Ok { data },
        Err(err) => APIResult::Err { error: err },
    }
}

pub fn apply_and_unwrap<T: Serialize, U: for<'a> Deserialize<'a>>(
    args: T,
    f: unsafe extern "C" fn(*const i8) -> *const i8,
    f_free: unsafe extern "C" fn(*const i8),
) -> Result<U, String> {
    let args_str = serde_json::to_string(&args).map_err(|err| err.to_string())?;
    let args_cstr = CString::new(args_str).map_err(|err| err.to_string())?;
    let raw_result = unsafe { (f)(args_cstr.as_ptr()) };
    if raw_result.is_null() {
        return Err("get_param_schema returns null".into());
    }

    let result = serde_json::de::from_str::<APIResult<U>>(&to_string(raw_result))
        .map_err(|err| err.to_string())?;
    unsafe { (f_free)(raw_result) };
    match result {
        APIResult::Ok { data } => Ok(data),
        APIResult::Err { error } => Err(error),
    }
}

pub type Handler = u64;

pub fn to_string(cchar: *const c_char) -> String {
    unsafe { CStr::from_ptr(cchar).to_string_lossy().into_owned() }
}

#[repr(usize)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

pub fn get_dylib_suffix() -> String {
    format!("-{OS}-{ARCH}{DLL_SUFFIX}")
}

pub fn get_dylib_stem_name(filename: &str) -> Option<String> {
    let suffix = get_dylib_suffix();
    if !filename.ends_with(&suffix) {
        None
    } else {
        let len = filename
            .chars()
            .count()
            .saturating_sub(suffix.chars().count());
        Some(filename.chars().take(len).collect())
    }
}

// https://doc.rust-lang.org/std/env/consts/index.html
pub fn resolve_dylib_name(name: &str) -> String {
    format!("{name}-{OS}-{ARCH}")
}
