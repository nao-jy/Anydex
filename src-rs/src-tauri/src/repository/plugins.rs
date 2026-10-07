use std::option::Option::Some;
use uuid::Uuid;

use crate::repository::json::{
    load,
    models::{Embedder, FileExtractor, FileSource, VectorStore},
    save,
};

pub async fn upsert_embedder(
    index_id: Uuid,
    embedder_name: String,
    params: serde_json::Value,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.embedder = Some(Embedder {
                name: embedder_name,
                params: params,
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn upsert_vector_store(
    index_id: Uuid,
    vector_store_name: String,
    params: serde_json::Value,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.vector_store = Some(VectorStore {
                name: vector_store_name,
                params: params,
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn upsert_file_extractor(
    index_id: Uuid,
    file_extractor_name: String,
    params: serde_json::Value,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.file_extractor = Some(FileExtractor {
                name: file_extractor_name,
                params: params,
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn upsert_file_source(
    index_id: Uuid,
    file_source_name: String,
    oauth_id: Option<Uuid>,
    params: serde_json::Value,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.file_source = Some(FileSource {
                name: file_source_name,
                params: params,
                oauth_id: oauth_id,
            });
            break;
        }
    }
    save(&userdata)
}
