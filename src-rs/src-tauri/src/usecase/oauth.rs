use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    repository::{
        json::{self, load, models::OAuthClient, save},
        sqlite,
    },
    server::{manager::end, oauth::client},
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

pub async fn test_oauth(client_config: OAuthClient) -> Result<(), String> {
    let scopes = client_config.scopes;
    let result = client::do_oauth_pkce(
        &client_config.client_id,
        client_config.client_secret.clone(),
        &client_config.authorization_endpoint,
        &client_config.token_endpoint,
        &scopes,
    )
    .await?;

    if let Some(revocation_endpoint) = client_config.revocation_endpoint {
        let _ = client::revoke_access_token(
            &client_config.client_id,
            client_config.client_secret.clone(),
            result.access_token.secret(),
            &revocation_endpoint,
        )
        .await;
        if let Some(refresh_token) = result.refresh_token {
            let _ = client::revoke_refresh_token(
                &client_config.client_id,
                client_config.client_secret,
                refresh_token.secret(),
                &revocation_endpoint,
            )
            .await;
        }
    }
    Ok(())
}

pub fn abort_oauth(client_id: &str) -> Result<(), String> {
    let _ = end(&client::get_server_name(client_id));
    Ok(())
}
