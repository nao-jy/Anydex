use std::sync::LazyLock;

use chrono::{Duration, Utc};
use keyring::Entry;
use tokio::sync::Mutex;

use crate::{
    consts,
    repository::{
        json::{load, models::OAuthClient},
        sqlite::models::oauth_authentication::Model,
    },
};

impl Model {
    fn get_access_token_username(&self) -> String {
        let id = &self.index_id;
        format!("{id}_acc")
    }
    fn get_refresh_token_username(&self) -> String {
        let id = &self.index_id;
        format!("{id}_ref")
    }
    async fn try_refresh(&mut self) -> Result<String, String> {
        let client = self.get_oauth_client();
        if let Some(cl) = &client {
            if let Some(refr) = self.load_refresh_token() {
                let result = cl.refresh(&refr).await?;
                let access_token = result.access_token.secret();
                self.save_access_token(access_token.to_string())?;
                self.expires_at = result.expires_at;
                return Ok(access_token.to_string());
            }
            return Err("refresh token not found".into());
        }
        Err("OAuth client not found".into())
    }
    pub async fn load_access_token(&mut self) -> Result<String, String> {
        static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
        let _guard = LOCK.lock().await;
        let entry = Entry::new(consts::APP_NAME, &self.get_access_token_username()).unwrap();
        let mut token = entry.get_password().map_or(None, |v| Some(v));
        let now = Utc::now();
        let expired = self
            .expires_at
            .is_some_and(|ex| ex <= (now + Duration::seconds(30)));
        if expired {
            token = Some(self.try_refresh().await?);
        }
        return token.ok_or("access token not found".into());
    }
    fn load_refresh_token(&self) -> Option<String> {
        let entry = Entry::new(consts::APP_NAME, &self.get_refresh_token_username()).unwrap();
        entry.get_password().map_or(None, |v| Some(v))
    }
    pub fn save_access_token(&self, token: String) -> Result<(), String> {
        let entry = Entry::new(consts::APP_NAME, &self.get_access_token_username()).unwrap();
        entry.set_password(&token).map_err(|err| err.to_string())
    }
    pub fn save_refresh_token(&self, token: String) -> Result<(), String> {
        let entry = Entry::new(consts::APP_NAME, &self.get_refresh_token_username()).unwrap();
        entry.set_password(&token).map_err(|err| err.to_string())
    }

    fn get_oauth_client(&self) -> Option<OAuthClient> {
        let userdata = load().ok()?;
        let client = userdata
            .oauth_clients
            .iter()
            .find(|v| v.id.to_string() == self.oauth_client_id);
        client.cloned()
    }

    pub async fn clear_tokens(&mut self) -> Result<(), String> {
        let client = self.get_oauth_client();
        let access_token = self.load_access_token().await;
        let refresh_token = self.load_refresh_token();

        if let Some(cl) = client {
            if let Ok(t) = access_token {
                cl.revoke_access_token(&t).await?;
            }
            if let Some(t) = refresh_token {
                cl.revoke_refresh_token(&t).await?;
            }
        }

        Entry::new(consts::APP_NAME, &self.get_access_token_username())
            .unwrap()
            .delete_credential()
            .map_err(|err| err.to_string())?;
        Entry::new(consts::APP_NAME, &self.get_refresh_token_username())
            .unwrap()
            .delete_credential()
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}
