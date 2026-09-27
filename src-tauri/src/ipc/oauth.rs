use sea_orm::sea_query::Expr;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
use sea_orm::QueryFilter;
use sea_orm::Set;
use sea_orm::Value;
use tauri::State;

use crate::dto::models::OAuthClient;
use crate::repository;

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn list_oauth_client(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<OAuthClient>, String> {
    let ret = repository::oauth::list_oauth_client(database.inner())
        .await
        .map_err(|err| err.to_string())?;
    Ok(ret)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn create_empty_oauth_client(
    database: State<'_, DatabaseConnection>,
) -> Result<OAuthClient, String> {
    let ret = repository::oauth::create_empty_oauth_client(database.inner())
        .await
        .map_err(|err| err.to_string())?;
    Ok(ret)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn update_oauth_client(
    data: OAuthClient,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    let model: Option<repository::models::oauth_client::Model> =
        repository::models::oauth_client::Entity::find_by_id(data.id)
            .one(database.inner())
            .await
            .map_err(|err| err.to_string())?;

    match model {
        Some(model_some) => {
            let mut active_model = model_some.into_active_model();
            active_model.name = Set(data.name);
            active_model.client_id = Set(data.client_id);
            active_model.authorization_endpoint = Set(data.authorization_endpoint);
            active_model.token_endpoint = Set(data.token_endpoint);
            active_model.refresh_endpoint = Set(data.refresh_endpoint);
            active_model.scopes = Set(data.scopes);

            active_model
                .update(database.inner())
                .await
                .map_err(|err| err.to_string())?;
            return Ok(());
        }
        None => {
            let id = data.id;
            return Err(format!("{id} not found"));
        }
    }
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn delete_oauth_client(
    oauth_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    let _result = repository::models::oauth_authentication::Entity::delete_many()
        .filter(repository::models::oauth_authentication::Column::OauthId.eq(oauth_id))
        .exec(database.inner())
        .await
        .map_err(|err| err.to_string())?;

    let _result = repository::models::file_source::Entity::update_many()
        .col_expr(
            repository::models::file_source::Column::OauthId,
            Expr::value(Value::Int(None)),
        )
        .filter(repository::models::file_source::Column::OauthId.eq(oauth_id))
        .exec(database.inner())
        .await
        .map_err(|err| err.to_string())?;

    let _result = repository::models::oauth_client::Entity::delete_by_id(oauth_id)
        .exec(database.inner())
        .await
        .map_err(|err| err.to_string())?;

    tracing::info!(oauth_id = oauth_id, "OAuth client deleted");
    Ok(())
}
