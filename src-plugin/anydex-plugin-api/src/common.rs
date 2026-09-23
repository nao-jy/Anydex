use std::env::consts::{ARCH, DLL_SUFFIX, OS};
use std::ffi::{c_char, CStr};

// common type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CSlice<T> {
    pub ptr: *const T,
    pub length: usize,
}

impl<T: Clone> Into<Vec<T>> for CSlice<T> {
    fn into(self) -> Vec<T> {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length).to_vec() }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CResult<T> {
    pub success: bool,
    pub data: T,
}

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
