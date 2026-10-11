use std::{
    fs::{self, File},
    io::BufWriter,
};

use crate::{path::get_dir, repository::json::models::UserData};

pub mod models;
pub mod oauth_client;

fn json_path() -> Result<String, String> {
    let app_data_dir = get_dir().app_data_dir;
    let json_path = app_data_dir
        .join("userdata.json")
        .to_string_lossy()
        .to_string();
    Ok(json_path)
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
