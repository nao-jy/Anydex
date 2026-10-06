//! SeaORM entities for the application database.
//!
//! `params`, `state`, and `scopes` are intentionally JSON values. SQLite stores
//! them as `TEXT`, while SeaORM keeps their application-side representation
//! type-safe enough to validate and query with SQLite's JSON functions.

pub mod embedder {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "embedders")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        /// One embedder configuration belongs to exactly one index.
        #[sea_orm(unique)]
        pub index_id: i32,
        // embedder plugin name
        pub name: String,
        pub params: Json,
        #[sea_orm(belongs_to, from = "index_id", to = "id")]
        pub index: BelongsTo<super::index::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod vector_store {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "vector_stores")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        /// One vector-store configuration belongs to exactly one index.
        #[sea_orm(unique)]
        pub index_id: i32,
        // vector store plugin name
        pub name: String,
        pub params: Json,
        #[sea_orm(belongs_to, from = "index_id", to = "id")]
        pub index: BelongsTo<super::index::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod schedule {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "schedules")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        /// One schedule belongs to exactly one indexer.
        #[sea_orm(unique)]
        pub indexer_id: i32,
        pub rfc5545_rrule: String,
        pub catchup_when_missed: bool,
        #[sea_orm(belongs_to, from = "indexer_id", to = "id")]
        pub indexer: BelongsTo<super::indexer::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod file_extractor {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "file_extractor")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        /// One file extractor belongs to exactly one indexer.
        #[sea_orm(unique)]
        pub indexer_id: i32,
        // file source plugin name
        pub name: String,
        pub params: Json,
        #[sea_orm(belongs_to, from = "indexer_id", to = "id")]
        pub indexer: BelongsTo<super::indexer::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod file_source {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "file_sources")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        /// One file source belongs to exactly one indexer.
        #[sea_orm(unique)]
        pub indexer_id: i32,
        // file source plugin name
        pub name: String,
        pub params: Json,
        pub oauth_id: Option<i32>,
        #[sea_orm(has_many)]
        pub files: HasMany<super::file::Entity>,
        #[sea_orm(belongs_to, from = "indexer_id", to = "id")]
        pub indexer: BelongsTo<super::indexer::Entity>,
        #[sea_orm(belongs_to, from = "oauth_id", to = "id")]
        pub oauth_client: BelongsTo<Option<super::oauth_client::Entity>>,
        #[sea_orm(has_one)]
        pub oauth_authentication: HasOne<super::oauth_authentication::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// OAuth credentials for a file source.
///
/// `file_source_id` is the primary key, so a file source can have zero or one
/// authentication record and can never have more than one.
pub mod oauth_authentication {
    use keyring::Entry;
    use sea_orm::entity::prelude::*;

    use crate::consts;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "oauth_authentications")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub file_source_id: i32,
        pub oauth_id: i32,
        pub access_token: String,
        pub refresh_token: Option<String>,
        pub expires_at: Option<DateTimeUtc>,
        #[sea_orm(belongs_to, from = "file_source_id", to = "id")]
        pub file_source: BelongsTo<super::file_source::Entity>,
        #[sea_orm(belongs_to, from = "oauth_id", to = "id")]
        pub oauth_client: BelongsTo<super::oauth_client::Entity>,
    }

    impl Model {
        pub fn load_access_token(&self) -> Option<String> {
            let entry = Entry::new(consts::APP_NAME, &self.access_token).unwrap();
            entry.get_password().map_or(None, |v| Some(v))
        }
        pub fn load_refresh_token(&self) -> Option<String> {
            match &self.refresh_token {
                Some(refr) => {
                    let entry = Entry::new(consts::APP_NAME, refr).unwrap();
                    entry.get_password().map_or(None, |v| Some(v))
                }
                None => None,
            }
        }
        pub fn save_token(token: String) -> Result<String, String> {
            let username = Uuid::new_v4().to_string();
            let entry = Entry::new(consts::APP_NAME, &username).unwrap();
            entry
                .set_password(&token)
                .map(|_| username)
                .map_err(|err| err.to_string())
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod indexer {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "indexers")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        /// An indexer is owned by one index and is not shared with another.
        pub index_id: i32,
        pub last_start: Option<DateTimeUtc>,
        pub last_success: Option<DateTimeUtc>,
        pub last_processed_files: Option<i32>,
        #[sea_orm(has_one)]
        pub file_source: HasOne<super::file_source::Entity>,
        #[sea_orm(has_one)]
        pub file_extractor: HasOne<super::file_extractor::Entity>,
        #[sea_orm(has_one)]
        pub schedule: HasOne<super::schedule::Entity>,
        #[sea_orm(belongs_to, from = "index_id", to = "id")]
        pub index: BelongsTo<super::index::Entity>,
        #[sea_orm(has_many)]
        pub tasks: HasMany<super::task::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod index {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "indexes")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub name: String,
        pub description: String,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
        #[sea_orm(default_value = false)]
        pub use_mcp_server: bool,
        #[sea_orm(has_one)]
        pub embedder: HasOne<super::embedder::Entity>,
        #[sea_orm(has_one)]
        pub vector_store: HasOne<super::vector_store::Entity>,
        #[sea_orm(has_many)]
        pub indexers: HasMany<super::indexer::Entity>,
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
        pub file_source_id: i32,
        pub name: String,
        /// Canonical URI in the file-source namespace. It is unique per source.
        #[sea_orm(unique_key = "file_source_uri")]
        pub uri: String,
        pub content_type: String,
        pub size: u64,
        pub updated_at: DateTimeUtc,
        pub file_hash: u64,
        #[sea_orm(belongs_to, from = "file_source_id", to = "id")]
        pub file_source: BelongsTo<super::file_source::Entity>,
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
        pub indexer_id: i32,
        pub task_type: String,
        pub status: String,
        pub started_at: Option<DateTimeUtc>,
        pub completed_at: Option<DateTimeUtc>,
        pub state: Json,
        #[sea_orm(belongs_to, from = "indexer_id", to = "id")]
        pub indexer: BelongsTo<super::indexer::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod oauth_client {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "oauth_clients")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub name: String,
        pub client_id: String,
        pub client_sercret: Option<String>,
        pub authorization_endpoint: String,
        pub token_endpoint: String,
        pub refresh_endpoint: Option<String>,
        pub revocation_endpoint: Option<String>,
        /// JSON array of OAuth scopes.
        pub scopes: Json,
        #[sea_orm(has_many)]
        pub authentications: HasMany<super::oauth_authentication::Entity>,
        #[sea_orm(has_many)]
        pub file_sources: HasMany<super::file_source::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
