use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

pub struct PathInfo {
    pub app_data_dir: String,
    pub app_cache_dir: String,
}
static PATH_INFO: Mutex<Option<PathInfo>> = Mutex::new(None);

pub fn init(app_data_dir: &str, app_cache_dir: &str) -> Result<(), String> {
    {
        let info = PathInfo {
            app_cache_dir: app_cache_dir.to_string(),
            app_data_dir: app_data_dir.to_string(),
        };
        let mut base_dir_mtx = PATH_INFO.lock().unwrap();
        *base_dir_mtx = Some(info);
    }
    std::fs::create_dir_all(app_data_dir).map_err(|err| err.to_string())?;
    std::fs::create_dir_all(app_cache_dir).map_err(|err| err.to_string())?;

    Ok(())
}

pub fn get_app_data_dir() -> Result<PathBuf, String> {
    PATH_INFO
        .lock()
        .map_err(|err| err.to_string())?
        .as_ref()
        .map(|v| Path::new(&v.app_data_dir).to_owned())
        .ok_or_else(|| "plugin base directory is not initialized".to_owned())
}

pub fn get_app_cache_dir() -> Result<PathBuf, String> {
    PATH_INFO
        .lock()
        .map_err(|err| err.to_string())?
        .as_ref()
        .map(|v| Path::new(&v.app_cache_dir).to_owned())
        .ok_or_else(|| "plugin base directory is not initialized".to_owned())
}
