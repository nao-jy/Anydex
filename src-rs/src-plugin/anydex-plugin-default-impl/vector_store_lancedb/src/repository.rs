use anydex_plugin_api_lib::vector_store::types::{Chunk, FileId, IndexId, SearchHit};

use futures::StreamExt;
use lancedb::arrow::arrow_array::Int64Array;
use lancedb::arrow::arrow_array::{
    types::Float32Type, Array, FixedSizeListArray, Float32Array, RecordBatch, StringArray,
};
use lancedb::arrow::arrow_schema::{DataType, Field, Schema};
use lancedb::connect;
use lancedb::connection::Connection;
use lancedb::index::scalar::FullTextSearchQuery;
use lancedb::index::{scalar::FtsIndexBuilder, Index, IndexType};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{DistanceType, Table};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::param::Param;

const TABLE_NAME: &str = "chunks";
const CONTENT_COLUMN: &str = "content";
const VECTOR_COLUMN: &str = "content_vector";

fn get_db_path(db_dir: &str, index_id: &str) -> PathBuf {
    let path = Path::new(db_dir).join("lancedb").join(index_id);
    return path;
}

fn init_db(db_dir: &str) -> Result<(), String> {
    let path = Path::new(db_dir).join("lancedb");
    std::fs::create_dir_all(path).map_err(|err| err.to_string())?;
    Ok(())
}

pub struct DB {
    pub connection: Connection,
    table: Table,
    pub param: Param,
    pub rt: Runtime,
}

impl DB {
    pub fn init_table(index_id: IndexId, db_dir: &str, param: &Param) -> Result<Self, String> {
        init_db(db_dir)?;

        if param.dimensions <= 0 {
            return Err("vector dimensions must be greater than zero".into());
        }
        let data = get_db_path(db_dir, &index_id).to_string_lossy().to_string();
        let rt = Runtime::new().map_err(|err| err.to_string())?;
        let db = rt
            .block_on(connect(&data).execute())
            .map_err(|err| err.to_string())?;
        let table_names = rt
            .block_on(db.table_names().execute())
            .map_err(|err| err.to_string())?;
        let table = if table_names.iter().any(|name| name == TABLE_NAME) {
            rt.block_on(db.open_table(TABLE_NAME).execute())
                .map_err(|err| err.to_string())?
        } else {
            let schema = Arc::new(Schema::new(vec![
                Field::new("chunk_json", DataType::Utf8, false),
                Field::new(CONTENT_COLUMN, DataType::Utf8, false),
                Field::new("index_id", DataType::Utf8, false),
                Field::new("file_id", DataType::Int64, false),
                Field::new(
                    VECTOR_COLUMN,
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", param.vector_type.into(), false)),
                        param.dimensions,
                    ),
                    false,
                ),
            ]));
            rt.block_on(db.create_empty_table(TABLE_NAME, schema).execute())
                .map_err(|err| err.to_string())?
        };
        let db = Self {
            connection: db,
            table,
            param: param.clone(),
            rt,
        };
        let row_count = db
            .rt
            .block_on(db.table.count_rows(None))
            .map_err(|err| err.to_string())?;
        if row_count > 0 {
            db.ensure_full_text_index()?;
        }
        Ok(db)
    }

    pub fn delete_by_file(&self, file_id: FileId) -> Result<(), String> {
        let predicate = format!("file_id = {file_id}");
        self.rt
            .block_on(self.table.delete(&predicate))
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        self.rt
            .block_on(self.table.delete("TRUE"))
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn insert(&self, chunks: Vec<Chunk>) -> Result<(), String> {
        if chunks.is_empty() {
            return Ok(());
        }

        for chunk in &chunks {
            if chunk.content_vector.len() != self.param.dimensions as usize {
                return Err(format!(
                    "chunk {} has vector dimension {}, expected {}",
                    chunk.id,
                    chunk.content_vector.len(),
                    self.param.dimensions
                ));
            }
        }

        let chunk_json = chunks
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;
        let content = chunks
            .iter()
            .map(|chunk| chunk.content.as_str())
            .collect::<Vec<_>>();
        let index_ids = chunks
            .iter()
            .map(|chunk| chunk.index_id.clone())
            .collect::<Vec<_>>();
        let file_ids = chunks.iter().map(|chunk| chunk.file_id).collect::<Vec<_>>();
        let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            chunks.iter().map(|chunk| {
                Some(
                    chunk
                        .content_vector
                        .iter()
                        .map(|value| Some(*value))
                        .collect::<Vec<_>>(),
                )
            }),
            self.param.dimensions,
        );
        let schema = self
            .rt
            .block_on(self.table.schema())
            .map_err(|err| err.to_string())?;
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(chunk_json)),
                Arc::new(StringArray::from(content)),
                Arc::new(StringArray::from(index_ids)),
                Arc::new(Int64Array::from(file_ids)),
                Arc::new(vectors),
            ],
        )
        .map_err(|err| err.to_string())?;
        self.rt
            .block_on(self.table.add(batch).execute())
            .map_err(|err| err.to_string())?;
        self.ensure_full_text_index()
    }

    pub fn search(
        &self,
        text: Option<String>,
        vec: Option<Vec<f32>>,
        target_file_ids: &[FileId],
        offset: i32,
        limit: i32,
    ) -> Result<Vec<SearchHit>, String> {
        if offset < 0 || limit < 0 {
            return Err("search offset and limit must not be negative".into());
        }
        if limit == 0 {
            return Ok(Vec::new());
        }
        if target_file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let text = text.filter(|text| !text.trim().is_empty());
        let vec = match vec {
            Some(vec) => {
                if vec.len() != self.param.dimensions as usize {
                    return Err(format!(
                        "query vector dimension {}, expected {}",
                        vec.len(),
                        self.param.dimensions
                    ));
                }
                Some(vec)
            }
            None if text.is_some() => None,
            None => return Err("text is required when the query embedding is absent".into()),
        };
        let full_text_only = vec.is_none();
        let hybrid_search = vec.is_some() && text.is_some();

        let filter = format!(
            "file_id IN ({})",
            target_file_ids
                .iter()
                .map(FileId::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        let batches = if let Some(vec) = vec {
            let query = self
                .table
                .query()
                .nearest_to(vec)
                .map_err(|err| err.to_string())?
                .column(VECTOR_COLUMN)
                .distance_type(DistanceType::Cosine)
                .only_if(&filter);
            let query = if let Some(text) = text {
                let hybrid_limit = offset
                    .checked_add(limit)
                    .ok_or("search offset and limit are too large")?;
                let text_query = FullTextSearchQuery::new(text)
                    .with_column(CONTENT_COLUMN.to_string())
                    .map_err(|err| err.to_string())?;
                query
                    .full_text_search(text_query)
                    .limit(hybrid_limit as usize)
            } else {
                query.offset(offset as usize).limit(limit as usize)
            };
            self.rt.block_on(query.execute())
        } else {
            let text = text.ok_or("text is required when the query embedding is absent")?;
            let text_query = FullTextSearchQuery::new(text)
                .with_column(CONTENT_COLUMN.to_string())
                .map_err(|err| err.to_string())?;
            let query = self
                .table
                .query()
                .only_if(&filter)
                .full_text_search(text_query)
                .offset(offset as usize)
                .limit(limit as usize);
            self.rt.block_on(query.execute())
        };
        let mut stream = batches.map_err(|err| err.to_string())?;
        let mut results = Vec::new();
        let mut skipped = 0;
        while let Some(batch) = self.rt.block_on(stream.next()) {
            let batch = batch.map_err(|err| err.to_string())?;
            let score_column = if full_text_only || hybrid_search {
                if full_text_only {
                    "_score"
                } else {
                    "_relevance_score"
                }
            } else {
                "_distance"
            };
            let chunks = batch
                .column_by_name("chunk_json")
                .ok_or("search result is missing chunk_json")?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or("search result chunk_json has an unexpected type")?;
            let distances = batch
                .column_by_name(score_column)
                .ok_or("search result is missing its score column")?
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or("search result score has an unexpected type")?;

            for row in 0..batch.num_rows() {
                if chunks.is_null(row) || distances.is_null(row) {
                    return Err("search result contains a null chunk or score".into());
                }
                if hybrid_search && skipped < offset as usize {
                    skipped += 1;
                    continue;
                }
                if results.len() >= limit as usize {
                    break;
                }
                let chunk =
                    serde_json::from_str(chunks.value(row)).map_err(|err| err.to_string())?;
                results.push(SearchHit {
                    chunk,
                    score: distances.value(row),
                });
            }
        }
        Ok(results)
    }

    fn ensure_full_text_index(&self) -> Result<(), String> {
        let indices = self
            .rt
            .block_on(self.table.list_indices())
            .map_err(|err| err.to_string())?;
        if indices.iter().any(|index| {
            index.index_type == IndexType::FTS
                && index.columns.iter().any(|column| column == CONTENT_COLUMN)
        }) {
            return Ok(());
        }

        self.rt
            .block_on(
                self.table
                    .create_index(&[CONTENT_COLUMN], Index::FTS(FtsIndexBuilder::default()))
                    .execute(),
            )
            .map_err(|err| err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::param::VectorType;

    use super::*;
    use serde_json::json;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_INDEX_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestDatabasePath(String);

    impl Drop for TestDatabasePath {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn test_chunk(id: String, index_id: String, file_id: i32) -> Chunk {
        serde_json::from_value(json!({
            "id": id,
            "fileId": file_id,
            "indexId": index_id,
            "content": format!("chunk {id}"),
            "contentVector": [1.0, 0.0],
        }))
        .unwrap()
    }

    fn test_index_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEST_INDEX_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{timestamp}-{counter}")
    }

    #[test]
    fn searches_only_target_files_and_supports_deletion() {
        let index_id = test_index_id();
        let path = format!("./data/{index_id}");
        assert!(
            !Path::new(&path).exists(),
            "test database path already exists"
        );
        let _cleanup = TestDatabasePath(path);

        let db = DB::init_table(
            index_id.clone(),
            "./",
            &Param {
                dimensions: 2,
                vector_type: VectorType::Float32,
            },
        )
        .unwrap();
        let mut aligned = test_chunk("1".into(), index_id.clone(), 100);
        aligned.content_vector = vec![10.0, 0.0];
        let mut other_file = test_chunk("2".into(), index_id.clone(), 200);
        other_file.content_vector = vec![1.0, 1.0];
        db.insert(vec![aligned, other_file]).unwrap();

        let nearest = db
            .search(None, Some(vec![1.0, 0.0]), &[100, 200], 0, 1)
            .unwrap();
        assert_eq!(nearest[0].chunk.id, "1");
        assert_eq!(nearest[0].chunk.index_id, index_id);

        let results = db
            .search(None, Some(vec![1.0, 0.0]), &[200], 0, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk.id, "2");
        assert!(db
            .search(None, Some(vec![1.0, 0.0]), &[], 0, 10)
            .unwrap()
            .is_empty());

        db.delete_by_file(100).unwrap();
        assert!(db
            .search(None, Some(vec![1.0, 0.0]), &[100, 200], 0, 10)
            .unwrap()
            .iter()
            .all(|hit| hit.chunk.id != "1"));

        db.delete_by_file(200).unwrap();
        assert!(db
            .search(None, Some(vec![1.0, 0.0]), &[100, 200], 0, 10)
            .unwrap()
            .is_empty());

        db.insert(vec![test_chunk("3".into(), index_id.clone(), 300)])
            .unwrap();
        db.clear().unwrap();
        assert!(db
            .search(None, Some(vec![1.0, 0.0]), &[300], 0, 10)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn uses_hybrid_search_when_text_is_present() {
        let index_id = test_index_id();
        let path = format!("./data/{index_id}");
        assert!(
            !Path::new(&path).exists(),
            "test database path already exists"
        );
        let _cleanup = TestDatabasePath(path);

        let db = DB::init_table(
            index_id.clone(),
            "./",
            &Param {
                dimensions: 2,
                vector_type: VectorType::Float32,
            },
        )
        .unwrap();
        let mut semantic_match = test_chunk("1".into(), index_id.clone(), 100);
        semantic_match.content = "unrelated content".into();
        semantic_match.content_vector = vec![1.0, 0.0];
        let mut text_match = test_chunk("2".into(), index_id, 100);
        text_match.content = "distinctive searchable phrase".into();
        text_match.content_vector = vec![0.0, 1.0];
        db.insert(vec![semantic_match, text_match]).unwrap();

        let results = db
            .search(
                Some("distinctive".into()),
                Some(vec![1.0, 0.0]),
                &[100],
                0,
                10,
            )
            .unwrap();
        let text_hit = results
            .iter()
            .find(|hit| hit.chunk.id == "2")
            .expect("hybrid search should include the full-text match");
        assert!(text_hit.score > 0.0);
        assert!(text_hit.score < 1.0);

        let next_page = db
            .search(
                Some("distinctive".into()),
                Some(vec![1.0, 0.0]),
                &[100],
                1,
                10,
            )
            .unwrap();
        assert_eq!(next_page.len(), 1);
    }

    #[test]
    fn searches_text_without_an_embedding() {
        let index_id = test_index_id();
        let path = format!("./data/{index_id}");
        assert!(
            !Path::new(&path).exists(),
            "test database path already exists"
        );
        let _cleanup = TestDatabasePath(path);

        let db = DB::init_table(
            index_id.clone(),
            "./",
            &Param {
                dimensions: 2,
                vector_type: VectorType::Float32,
            },
        )
        .unwrap();
        let mut matching = test_chunk("1".into(), index_id.clone(), 100);
        matching.content = "distinctive exact phrase".into();
        matching.content_vector = vec![0.0, 1.0];
        let mut unrelated = test_chunk("2".into(), index_id.clone(), 100);
        unrelated.content = "unrelated content".into();
        unrelated.content_vector = vec![1.0, 0.0];
        let mut other_file = test_chunk("3".into(), index_id.clone(), 200);
        other_file.content = "distinctive phrase in another file".into();
        db.insert(vec![matching, unrelated, other_file]).unwrap();

        let results = db
            .search(Some("distinctive".into()), None, &[100], 0, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk.id, "1");
        assert!(results[0].score > 0.0);

        let next_page = db
            .search(Some("distinctive".into()), None, &[200], 0, 10)
            .unwrap();
        assert_eq!(next_page.len(), 1);
        assert_eq!(next_page[0].chunk.id, "3");
    }

    #[test]
    fn rejects_textless_search_without_an_embedding() {
        let index_id = test_index_id();
        let path = format!("./data/{index_id}");
        assert!(
            !Path::new(&path).exists(),
            "test database path already exists"
        );
        let _cleanup = TestDatabasePath(path);
        let db = DB::init_table(
            index_id,
            "./",
            &Param {
                dimensions: 2,
                vector_type: VectorType::Float32,
            },
        )
        .unwrap();

        assert!(db.search(None, None, &[100], 0, 10).is_err());
        assert!(db.search(Some("  ".into()), None, &[100], 0, 10).is_err());
    }
}
