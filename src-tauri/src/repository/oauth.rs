use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::{dto::models::OAuthClient, repository::models};

pub async fn load_oauth_client(
    database: &DatabaseConnection,
    file_source_id: i32,
) -> Result<Option<OAuthClient>, DbErr> {
    let Some(authentication) = models::oauth_authentication::Entity::find_by_id(file_source_id)
        .one(database)
        .await?
    else {
        return Ok(None);
    };
    let client = models::oauth_client::Entity::find_by_id(authentication.oauth_id)
        .one(database)
        .await?;

    match client {
        Some(cl) => Ok(Some(OAuthClient {
            id: cl.id,
            name: cl.name,
            client_id: cl.client_id,
            authorization_endpoint: cl.authorization_endpoint,
            token_endpoint: cl.token_endpoint,
            refresh_endpoint: cl.refresh_endpoint,
            scopes: cl.scopes,
        })),
        None => Ok(None),
    }
}
