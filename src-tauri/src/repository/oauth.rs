use crate::{dto::models::OAuthClient, repository::models};
use sea_orm::ActiveModelTrait;
use sea_orm::TransactionTrait;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, Order, Set};

pub async fn list_oauth_client(database: &DatabaseConnection) -> Result<Vec<OAuthClient>, DbErr> {
    let models = models::oauth_client::Entity::find()
        .order_by_id(Order::Asc)
        .all(database)
        .await?
        .into_iter()
        .map(|cl| OAuthClient {
            id: cl.id,
            name: cl.name,
            client_id: cl.client_id,
            client_secret: cl.client_sercret,
            authorization_endpoint: cl.authorization_endpoint,
            token_endpoint: cl.token_endpoint,
            refresh_endpoint: cl.refresh_endpoint,
            revocation_endpoint: cl.revocation_endpoint,
            scopes: cl.scopes,
        })
        .collect();
    Ok(models)
}

pub async fn create_empty_oauth_client(
    database: &DatabaseConnection,
) -> Result<OAuthClient, DbErr> {
    let transaction = database.begin().await?;
    let name: String = "新しいOAuth".into();
    let scope: Vec<String> = Vec::new();

    let model = models::oauth_client::ActiveModel {
        name: Set(name),
        client_id: Set("".into()),
        authorization_endpoint: Set("".into()),
        token_endpoint: Set("".into()),
        refresh_endpoint: Set(None),
        scopes: Set(serde_json::Value::from(scope)),
        ..Default::default()
    }
    .insert(&transaction)
    .await?;

    transaction.commit().await?;
    Ok(OAuthClient {
        id: model.id,
        name: model.name,
        client_id: model.client_id,
        client_secret: model.client_sercret,
        authorization_endpoint: model.authorization_endpoint,
        token_endpoint: model.token_endpoint,
        refresh_endpoint: model.refresh_endpoint,
        revocation_endpoint: model.revocation_endpoint,
        scopes: model.scopes,
    })
}

pub async fn load_oauth_client(
    database: &DatabaseConnection,
    oauth_id: i32,
) -> Result<Option<OAuthClient>, DbErr> {
    let client = models::oauth_client::Entity::find_by_id(oauth_id)
        .one(database)
        .await?;

    match client {
        Some(cl) => Ok(Some(OAuthClient {
            id: cl.id,
            name: cl.name,
            client_id: cl.client_id,
            client_secret: cl.client_sercret,
            authorization_endpoint: cl.authorization_endpoint,
            token_endpoint: cl.token_endpoint,
            refresh_endpoint: cl.refresh_endpoint,
            revocation_endpoint: cl.revocation_endpoint,
            scopes: cl.scopes,
        })),
        None => Ok(None),
    }
}
