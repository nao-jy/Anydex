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
        pub file_hash: u64,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod task {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tasks")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub index_id: String,
        pub task_type: String,
        pub status: String,
        pub started_at: Option<DateTimeUtc>,
        pub completed_at: Option<DateTimeUtc>,
        pub state: Json,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
