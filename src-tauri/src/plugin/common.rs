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
pub struct CPair<T, U> {
    pub key: T,
    pub value: U,
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
