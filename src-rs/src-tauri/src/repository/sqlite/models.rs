pub mod oauth_authentication {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "oauth_authentications")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub index_id: String,
        pub oauth_client_id: String,
        pub expires_at: Option<DateTimeUtc>,
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
        pub id: i64,
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
        pub size: i64,
        pub updated_at: Option<DateTimeUtc>,
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
        pub id: i64,
        pub index_id: String,
        pub job_type: String,
        pub payload: Json,
        pub status: String,
        pub progress_info: String,
        pub priority: i32,
        pub attempts: i32,
        pub max_attempts: i32,
        pub run_at: DateTimeUtc,
        pub last_error: Option<String>,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod indexing_history {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "indexing_histories")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub index_id: String,
        pub started_at: DateTimeUtc,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
