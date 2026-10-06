use anydex_plugin_api_lib::vector_store::types::{
    Chunk, FileId, FilterCondition, IndexId, IndexerId, SearchHit,
};

use futures::StreamExt;
use lancedb::arrow::arrow_array::{
    types::Float32Type, Array, ArrayRef, FixedSizeListArray, Float32Array, Int32Array, Int64Array,
    RecordBatch, StringArray,
};
use lancedb::arrow::arrow_schema::{DataType, Field, Schema};
use lancedb::connect;
use lancedb::connection::Connection;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{DistanceType, Table};
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::param::Param;

const TABLE_NAME: &str = "chunks";
const VECTOR_COLUMN: &str = "content_vector";

pub fn init_db() -> Result<(), String> {
    std::fs::create_dir_all("./data").map_err(|err| err.to_string())?;
    Ok(())
}

pub struct DB {
    pub connection: Connection,
    table: Table,
    pub param: Param,
    pub rt: Runtime,
}

impl DB {
    pub fn init_table(index_id: IndexId, param: &Param) -> Result<Self, String> {
        if param.dimensions <= 0 {
            return Err("vector dimensions must be greater than zero".into());
        }
        let data = format!("./data/{index_id}");
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
                Field::new("indexer_id", DataType::Int32, false),
                Field::new("file_id", DataType::Int32, false),
                Field::new("file_dir", DataType::Utf8, false),
                Field::new("file_name", DataType::Utf8, false),
                Field::new("content_type", DataType::Utf8, false),
                Field::new("file_ext", DataType::Utf8, false),
                Field::new("created_at_millis", DataType::Int64, false),
                Field::new(
                    VECTOR_COLUMN,
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", DataType::Float32, false)),
                        param.dimensions,
                    ),
                    false,
                ),
            ]));
            rt.block_on(db.create_empty_table(TABLE_NAME, schema).execute())
                .map_err(|err| err.to_string())?
        };
        Ok(Self {
            connection: db,
            table,
            param: param.clone(),
            rt,
        })
    }

    pub fn delete_by_file(&self, file_id: FileId) -> Result<(), String> {
        let predicate = format!("file_id = {file_id}");
        self.rt
            .block_on(self.table.delete(&predicate))
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn delete_by_indexer(&self, indexer_id: IndexerId) -> Result<(), String> {
        let predicate = format!("indexer_id = {indexer_id}");
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
        let indexer_ids = chunks
            .iter()
            .map(|chunk| chunk.indexer_id)
            .collect::<Vec<_>>();
        let file_ids = chunks.iter().map(|chunk| chunk.file_id).collect::<Vec<_>>();
        let file_dirs = chunks
            .iter()
            .map(|chunk| chunk.file_dir.clone())
            .collect::<Vec<_>>();
        let file_names = chunks
            .iter()
            .map(|chunk| chunk.file_name.clone())
            .collect::<Vec<_>>();
        let content_types = chunks
            .iter()
            .map(|chunk| chunk.content_type.clone())
            .collect::<Vec<_>>();
        let file_exts = chunks
            .iter()
            .map(|chunk| {
                Path::new(&chunk.file_name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or_default()
                    .to_lowercase()
            })
            .collect::<Vec<_>>();
        let created_at_millis = chunks
            .iter()
            .map(|chunk| chunk.created_at.timestamp_millis())
            .collect::<Vec<_>>();
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
        let columns: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(chunk_json)),
            Arc::new(Int32Array::from(indexer_ids)),
            Arc::new(Int32Array::from(file_ids)),
            Arc::new(StringArray::from(file_dirs)),
            Arc::new(StringArray::from(file_names)),
            Arc::new(StringArray::from(content_types)),
            Arc::new(StringArray::from(file_exts)),
            Arc::new(Int64Array::from(created_at_millis)),
            Arc::new(vectors),
        ];
        let schema = self
            .rt
            .block_on(self.table.schema())
            .map_err(|err| err.to_string())?;
        let batch = RecordBatch::try_new(schema, columns).map_err(|err| err.to_string())?;
        self.rt
            .block_on(self.table.add(batch).execute())
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn search(
        &self,
        vec: &[f32],
        cond: &FilterCondition,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<SearchHit>, String> {
        if vec.len() != self.param.dimensions as usize {
            return Err(format!(
                "query vector dimension {}, expected {}",
                vec.len(),
                self.param.dimensions
            ));
        }
        if offset < 0 || limit < 0 {
            return Err("search offset and limit must not be negative".into());
        }
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut filters = vec![
            format!("created_at_millis >= {}", cond.from.timestamp_millis()),
            format!("created_at_millis <= {}", cond.to.timestamp_millis()),
        ];
        if !cond.dirs.is_empty() {
            filters.push(format!(
                "file_dir IN ({})",
                sql_string_values(cond.dirs.iter().map(String::as_str))
            ));
        }
        if !cond.content_type.is_empty() {
            filters.push(format!(
                "content_type IN ({})",
                sql_string_values(cond.content_type.iter().map(String::as_str))
            ));
        }
        if !cond.file_name.is_empty() {
            filters.push(format!(
                "file_name IN ({})",
                sql_string_values(cond.file_name.iter().map(String::as_str))
            ));
        }
        if !cond.file_ext.is_empty() {
            filters.push(format!(
                "file_ext IN ({})",
                sql_string_values(
                    cond.file_ext
                        .iter()
                        .map(|ext| ext.trim_start_matches('.').to_lowercase())
                        .collect::<Vec<_>>()
                        .iter()
                        .map(String::as_str)
                )
            ));
        }

        let filter = filters.join(" AND ");
        let batches = self.rt.block_on(async {
            self.table
                .query()
                .nearest_to(vec)?
                .column(VECTOR_COLUMN)
                .distance_type(DistanceType::Cosine)
                .only_if(filter)
                .offset(offset as usize)
                .limit(limit as usize)
                .execute()
                .await
        });
        let mut stream = batches.map_err(|err| err.to_string())?;
        let mut results = Vec::new();
        while let Some(batch) = self.rt.block_on(stream.next()) {
            let batch = batch.map_err(|err| err.to_string())?;
            let chunks = batch
                .column_by_name("chunk_json")
                .ok_or("search result is missing chunk_json")?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or("search result chunk_json has an unexpected type")?;
            let distances = batch
                .column_by_name("_distance")
                .ok_or("search result is missing _distance")?
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or("search result _distance has an unexpected type")?;

            for row in 0..batch.num_rows() {
                if chunks.is_null(row) || distances.is_null(row) {
                    return Err("search result contains a null chunk or distance".into());
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
}

fn sql_string_values<'a>(values: impl Iterator<Item = &'a str>) -> String {
    values
        .map(|value| format!("'{}'", value.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDatabasePath(String);

    impl Drop for TestDatabasePath {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn test_chunk(id: i32, indexer_id: i32, file_id: i32, directory: &str) -> Chunk {
        serde_json::from_value(json!({
            "id": id,
            "index_id": 731,
            "indexer_id": indexer_id,
            "file_id": file_id,
            "uri": format!("file:///{directory}/file-{id}.txt"),
            "file_name": format!("file-{id}.txt"),
            "file_dir": directory,
            "content_type": "text/plain",
            "content": format!("chunk {id}"),
            "content_vector": [1.0, 0.0],
            "metadata": {},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }))
        .unwrap()
    }

    fn test_filter() -> FilterCondition {
        serde_json::from_value(json!({
            "dirs": [],
            "content_type": [],
            "file_name": [],
            "file_ext": [],
            "from": "2025-01-01T00:00:00Z",
            "to": "2027-01-01T00:00:00Z"
        }))
        .unwrap()
    }

    #[test]
    fn supports_insert_search_and_deletion() {
        init_db().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i32;
        let index_id = timestamp.saturating_abs().max(1);
        let path = format!("./data/{index_id}");
        assert!(
            !Path::new(&path).exists(),
            "test database path already exists"
        );
        let _cleanup = TestDatabasePath(path);

        let db = DB::init_table(index_id, &Param { dimensions: 2 }).unwrap();
        let mut aligned = test_chunk(1, 10, 100, "/docs");
        aligned.content_vector = vec![10.0, 0.0];
        let mut closer_by_l2 = test_chunk(2, 20, 200, "/notes");
        closer_by_l2.content_vector = vec![1.0, 1.0];
        db.insert(vec![aligned, closer_by_l2]).unwrap();

        let nearest = db.search(&[1.0, 0.0], &test_filter(), 0, 1).unwrap();
        assert_eq!(nearest[0].chunk.id, 1);

        let mut filter = test_filter();
        filter.dirs = vec!["/docs".into()];
        filter.file_ext = vec![".TXT".into()];
        let results = db.search(&[1.0, 0.0], &filter, 0, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk.id, 1);

        db.delete_by_file(100).unwrap();
        assert!(db
            .search(&[1.0, 0.0], &test_filter(), 0, 10)
            .unwrap()
            .iter()
            .all(|hit| hit.chunk.id != 1));

        db.delete_by_indexer(20).unwrap();
        assert!(db
            .search(&[1.0, 0.0], &test_filter(), 0, 10)
            .unwrap()
            .is_empty());

        db.insert(vec![test_chunk(3, 30, 300, "/docs")]).unwrap();
        db.clear().unwrap();
        assert!(db
            .search(&[1.0, 0.0], &test_filter(), 0, 10)
            .unwrap()
            .is_empty());
    }
}
