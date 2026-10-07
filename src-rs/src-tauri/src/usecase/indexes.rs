use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use crate::repository::{
    json::{
        load,
        models::{Index, Schedule},
        save,
    },
    sqlite::{self},
};

pub async fn list_indexes() -> Result<Vec<Index>, String> {
    let userdata = load()?;
    Ok(userdata.indexes)
}

pub async fn create_empty_index() -> Result<(), String> {
    let mut userdata = load()?;
    userdata.indexes.push(Index::default());
    save(&userdata)
}

pub async fn rename_index(index_id: Uuid, name: String) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.name = name;
            break;
        }
    }
    save(&userdata)
}

pub async fn put_description(index_id: Uuid, description: String) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.description = description;
            break;
        }
    }
    save(&userdata)
}

pub async fn set_index_mcp_server(index_id: Uuid, enabled: bool) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.use_mcp_server = enabled;
            break;
        }
    }
    save(&userdata)
}

pub async fn delete_index(index_id: Uuid, database: &DatabaseConnection) -> Result<(), String> {
    let transaction = database.begin().await.map_err(|err| err.to_string())?;

    let result = sqlite::models::oauth_authentication::Entity::find_by_id(index_id.to_string())
        .one(&transaction)
        .await
        .map_err(|err| err.to_string())?;
    if let Some(auth) = result {
        auth.clear_tokens()?;
    }

    let _result = sqlite::models::oauth_authentication::Entity::delete_by_id(index_id.to_string())
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let _result = sqlite::models::file::Entity::delete_many()
        .filter(sqlite::models::file::Column::IndexId.eq(index_id.to_string()))
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let _result = sqlite::models::task::Entity::delete_many()
        .filter(sqlite::models::task::Column::IndexId.eq(index_id.to_string()))
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    transaction.commit().await.map_err(|err| err.to_string())?;

    let mut userdata = load()?;
    userdata.indexes = userdata
        .indexes
        .iter()
        .filter(|index| index.id != index_id)
        .cloned()
        .collect();
    save(&userdata)
}

pub async fn upsert_schedule(index_id: Uuid, schedule: Option<Schedule>) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.schedule = schedule;
            break;
        }
    }
    save(&userdata)
}
