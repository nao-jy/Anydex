use std::{path::PathBuf, sync::Mutex};

#[derive(Debug, Clone)]
pub struct PathInfo {
    pub app_data_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_dl_dir: PathBuf,
}
static PATH_INFO: Mutex<Option<PathInfo>> = Mutex::new(None);

pub fn init(app_data_dir: &str, app_cache_dir: &str, app_dl_dir: &str) -> Result<(), String> {
    {
        let info = PathInfo {
            app_cache_dir: PathBuf::from(app_cache_dir),
            app_data_dir: PathBuf::from(app_data_dir),
            app_dl_dir: PathBuf::from(app_dl_dir),
        };
        let mut base_dir_mtx = PATH_INFO.lock().unwrap();
        *base_dir_mtx = Some(info);
    }
    std::fs::create_dir_all(app_data_dir).map_err(|err| err.to_string())?;
    std::fs::create_dir_all(app_cache_dir).map_err(|err| err.to_string())?;

    Ok(())
}

pub fn get_dir() -> PathInfo {
    PATH_INFO
        .lock()
        .map_err(|err| err.to_string())
        .unwrap()
        .as_ref()
        .cloned()
        .unwrap()
}
