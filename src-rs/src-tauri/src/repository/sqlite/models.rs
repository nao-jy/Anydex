pub mod oauth_authentication {
    use keyring::Entry;
    use sea_orm::entity::prelude::*;

    use crate::consts;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "oauth_authentications")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub index_id: String,
        pub oauth_client_id: String,
        pub expires_at: Option<DateTimeUtc>,
    }

    impl Model {
        fn get_access_token_username(&self) -> String {
            let id = &self.index_id;
            format!("{id}_acc")
        }
        fn get_refresh_token_username(&self) -> String {
            let id = &self.index_id;
            format!("{id}_ref")
        }
        pub fn load_access_token(&self) -> Option<String> {
            let entry = Entry::new(consts::APP_NAME, &self.get_access_token_username()).unwrap();
            entry.get_password().map_or(None, |v| Some(v))
        }
        pub fn load_refresh_token(&self) -> Option<String> {
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
        pub fn clear_tokens(&self) -> Result<(), String> {
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

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod file {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "files")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique_key = "file_source_uri")]
        pub index_id: String,
        pub name: String,
        /// Canonical URI in the file-source namespace. It is unique per source.
        #[sea_orm(unique_key = "file_source_uri")]
        pub file_uri: String,
        pub file_path: String,
        pub file_dir: String,
        pub file_name: String,
        pub content_type: String,
        pub size: u64,
        pub updated_at: DateTimeUtc,
        pub file_hash: Option<String>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod job {
    use sea_orm::entity::prelude::*;

    pub const STATUS_PENDING: &str = "pending";
    pub const STATUS_RUNNING: &str = "running";
    pub const STATUS_DONE: &str = "done";
    pub const STATUS_FAILED: &str = "failed";
    pub const STATUS_DEAD: &str = "dead";

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "jobs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u64,
        pub index_id: String,
        pub job_type: String,
        pub payload: Json,
        pub status: String,
        pub priority: i32,
        pub attempts: i32,
        pub max_attempts: i32,
        pub run_at: DateTimeUtc,
        pub locked_until: Option<DateTimeUtc>,
        pub last_error: Option<String>,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
