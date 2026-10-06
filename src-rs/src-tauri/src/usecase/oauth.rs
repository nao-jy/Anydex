use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, Set, Value,
};

use crate::{
    dto::models::OAuthClient,
    repository,
    server::{manager::end, oauth::client},
};

pub async fn list_oauth_clients(database: &DatabaseConnection) -> Result<Vec<OAuthClient>, String> {
    repository::oauth::list_oauth_client(database)
        .await
        .map_err(|err| err.to_string())
}

pub async fn create_empty_oauth_client(
    database: &DatabaseConnection,
) -> Result<OAuthClient, String> {
    repository::oauth::create_empty_oauth_client(database)
        .await
        .map_err(|err| err.to_string())
}

pub async fn update_oauth_client(
    data: OAuthClient,
    database: &DatabaseConnection,
) -> Result<(), String> {
    let model = repository::models::oauth_client::Entity::find_by_id(data.id)
        .one(database)
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("{} not found", data.id))?;

    let mut active_model = model.into_active_model();
    active_model.name = Set(data.name);
    active_model.client_id = Set(data.client_id);
    active_model.authorization_endpoint = Set(data.authorization_endpoint);
    active_model.token_endpoint = Set(data.token_endpoint);
    active_model.refresh_endpoint = Set(data.refresh_endpoint);
    active_model.scopes = Set(data.scopes);
    active_model.revocation_endpoint = Set(data.revocation_endpoint);
    active_model.client_sercret = Set(data.client_secret);
    active_model
        .update(database)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn delete_oauth_client(
    oauth_id: i32,
    database: &DatabaseConnection,
) -> Result<(), String> {
    repository::models::oauth_authentication::Entity::delete_many()
        .filter(repository::models::oauth_authentication::Column::OauthId.eq(oauth_id))
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;

    repository::models::file_source::Entity::update_many()
        .col_expr(
            repository::models::file_source::Column::OauthId,
            Expr::value(Value::Int(None)),
        )
        .filter(repository::models::file_source::Column::OauthId.eq(oauth_id))
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;

    repository::models::oauth_client::Entity::delete_by_id(oauth_id)
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;

    tracing::info!(oauth_id, "OAuth client deleted");
    Ok(())
}

pub async fn test_oauth(client_config: OAuthClient) -> Result<(), String> {
    let scopes = client_config
        .scopes
        .as_array()
        .ok_or_else(|| "OAuth scopes must be an array".to_owned())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "OAuth scopes must contain only strings".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
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
