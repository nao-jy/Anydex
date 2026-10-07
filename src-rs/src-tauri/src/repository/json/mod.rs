use std::{
    fs::{self, File},
    io::BufWriter,
    sync::Mutex,
};

use crate::repository::json::models::UserData;

pub mod models;

static JSON_PATH: Mutex<Option<String>> = Mutex::new(None);

fn json_path() -> Result<String, String> {
    let path = match JSON_PATH.lock() {
        Ok(mtx) => mtx.as_ref().cloned(),
        Err(poisoned) => {
            let mtx = poisoned.into_inner();
            JSON_PATH.clear_poison();
            mtx.as_ref().cloned()
        }
    };

    path.ok_or_else(|| "not initialized".to_string())
}

pub fn initialize(json_path: &str) -> Result<(), String> {
    let mut mtx = JSON_PATH
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *mtx = Some(String::from(json_path));
    Ok(())
}

pub fn save(userdata: &UserData) -> Result<(), String> {
    let json_path = json_path()?;
    let file = File::create(json_path).map_err(|err| err.to_string())?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, userdata).map_err(|err| err.to_string())?;
    Ok(())
}

pub fn load() -> Result<UserData, String> {
    let json_path = json_path()?;
    let data = fs::read_to_string(json_path);
    match data {
        Ok(s) => serde_json::from_str(&s).or(Ok(UserData::default())),
        Err(_) => Ok(UserData::default()),
    }
}
