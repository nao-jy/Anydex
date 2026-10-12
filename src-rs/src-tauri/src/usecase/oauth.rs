use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};
use uuid::Uuid;

use crate::{
    repository::{
        json::{self, load, models::OAuthClient, save},
        sqlite::{self, models::oauth_authentication},
    },
    server::manager::end,
};

pub async fn list_oauth_clients() -> Result<Vec<OAuthClient>, String> {
    let userdata = load()?;
    Ok(userdata.oauth_clients)
}

pub async fn create_empty_oauth_client() -> Result<(), String> {
    let mut userdata = load()?;
    userdata
        .oauth_clients
        .push(json::models::OAuthClient::default());
    save(&userdata)
}

pub async fn update_oauth_client(data: OAuthClient) -> Result<(), String> {
    let mut userdata = load()?;
    for client in userdata.oauth_clients.iter_mut() {
        if client.id == data.id {
            *client = data;
            break;
        }
    }
    save(&userdata)
}

pub async fn delete_oauth_client(
    oauth_id: Uuid,
    database: &DatabaseConnection,
) -> Result<(), String> {
    sqlite::models::oauth_authentication::Entity::delete_many()
        .filter(
            sqlite::models::oauth_authentication::Column::OauthClientId.eq(oauth_id.to_string()),
        )
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;

    let mut userdata = load()?;
    userdata.oauth_clients = userdata
        .oauth_clients
        .iter()
        .filter(|v| v.id != oauth_id)
        .cloned()
        .collect();
    save(&userdata)
}

pub async fn delete_oauth_authentication(
    index_id: Uuid,
    database: &DatabaseConnection,
) -> Result<(), String> {
    sqlite::models::oauth_authentication::Entity::delete_many()
        .filter(sqlite::models::oauth_authentication::Column::IndexId.eq(index_id.to_string()))
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn test_oauth(client_config: OAuthClient) -> Result<(), String> {
    let result = client_config.do_oauth_pkce().await?;

    let _ = client_config
        .revoke_access_token(result.access_token.secret())
        .await;
    if let Some(refresh_token) = result.refresh_token {
        let _ = client_config
            .revoke_refresh_token(refresh_token.secret())
            .await;
    }
    Ok(())
}

pub async fn do_oauth_authentication_if_not(
    index_id: Uuid,
    client_config: OAuthClient,
    force: bool,
    database: &DatabaseConnection,
) -> Result<(), String> {
    let auth = get_authentication(index_id, client_config.id, database).await?;
    if auth.is_none() && !force {
        return Ok(());
    }

    let result = client_config.do_oauth_pkce().await?;
    let model = sqlite::models::oauth_authentication::Model {
        index_id: index_id.to_string(),
        oauth_client_id: client_config.id.to_string(),
        expires_at: result.expires_at,
    };
    model.save_access_token(result.access_token.secret().to_owned())?;
    if let Some(refr) = &result.refresh_token {
        model.save_refresh_token(refr.secret().to_owned())?;
    }
    let _ = delete_authentication(index_id, database).await;
    sqlite::models::oauth_authentication::Entity::insert(model.into_active_model())
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

pub fn abort_oauth(client_id: &str) -> Result<(), String> {
    let _ = end(&OAuthClient::get_server_name(client_id));
    Ok(())
}

pub async fn delete_authentication(
    index_id: Uuid,
    database: &DatabaseConnection,
) -> Result<(), String> {
    let mut data = sqlite::models::oauth_authentication::Entity::find_by_id(index_id.to_string())
        .one(database)
        .await
        .map_err(|err| err.to_string())?;
    if let Some(v) = data.as_mut() {
        let _ = v.clear_tokens().await;
        sqlite::models::oauth_authentication::Entity::delete_by_id(index_id.to_string())
            .exec(database)
            .await
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

pub async fn get_authentication(
    index_id: Uuid,
    oauth_client_id: Uuid,
    database: &DatabaseConnection,
) -> Result<Option<oauth_authentication::Model>, String> {
    let data = sqlite::models::oauth_authentication::Entity::find_by_id(index_id.to_string())
        .one(database)
        .await
        .map_err(|err| err.to_string())?;
    if let Some(v) = &data {
        if v.oauth_client_id != oauth_client_id.to_string() {
            delete_authentication(index_id, database).await?;
            return Ok(None);
        }
    }
    return Ok(data);
}
