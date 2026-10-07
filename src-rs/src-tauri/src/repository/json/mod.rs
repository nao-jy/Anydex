use std::{
    fs::{self, File},
    io::BufWriter,
    sync::Mutex,
};

use crate::repository::json::models::UserData;

pub mod models;

static JSON_PATH: Mutex<Option<String>> = Mutex::new(None);

pub fn initialize(json_path: &str) -> Result<(), String> {
    {
        let mut mtx = JSON_PATH.lock().unwrap();
        *mtx = Some(String::from(json_path));
    }
    Ok(())
}

pub fn save(userdata: &UserData) -> Result<(), String> {
    let mtx = JSON_PATH.lock().unwrap();
    let json_path = mtx.as_ref().ok_or("not initialized")?.clone();
    let file = File::create(json_path).map_err(|err| err.to_string())?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, userdata).map_err(|err| err.to_string())?;
    Ok(())
}

pub fn load() -> Result<UserData, String> {
    let mtx = JSON_PATH.lock().unwrap();
    let json_path = mtx.as_ref().ok_or("not initialized")?.clone();
    let data = fs::read_to_string(json_path).expect("Unable to read file");
    let p: UserData = serde_json::from_str(&data).expect("JSON was not well-formatted");
    Ok(p)
}
