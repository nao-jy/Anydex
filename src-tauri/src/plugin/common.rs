use std::ffi::{c_char, CStr};

use tracing::{debug, error, info, trace, warn};

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

#[repr(u32)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}
pub unsafe extern "C" fn host_log(log_level: LogLevel, message: *const c_char) {
    let message = to_string(message);
    match log_level {
        LogLevel::Error => error!("{}", message),
        LogLevel::Warn => warn!("{}", message),
        LogLevel::Info => info!("{}", message),
        LogLevel::Debug => debug!("{}", message),
        LogLevel::Trace => trace!("{}", message),
    }
}
