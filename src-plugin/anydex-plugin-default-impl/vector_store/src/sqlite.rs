use std::{str::FromStr, sync::Mutex};

use anydex_plugin_api_lib::vector_store::interface::{FileId, IndexId};
use rusqlite::Connection;

use crate::param::Param;

static DB: Mutex<Option<Connection>> = Mutex::new(None);

pub struct Chunk {
    pub id: i32,
    pub index_id: i32,
    pub indexer_id: i32,
    pub file_id: i32,
    pub uri: String,
    pub content_type: String,
    pub content: Option<String>,
    pub content_vector: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
    pub created_at_utc: i64,
    pub updated_at_utc: i64,
}

pub fn init_sqlite(sqlite_extension_path: &str) -> Result<(), String> {
    let conn = Connection::open("vector-store.db").map_err(|err| err.to_string())?;
    unsafe {
        conn.load_extension_enable()
            .map_err(|err| err.to_string())?;
    }

    // load vector extension
    conn.execute("SELECT load_extension(?)", [sqlite_extension_path])
        .map_err(|err| err.to_string())?;

    let mut db_mtx = DB.lock().unwrap();
    (*db_mtx) = Some(conn);
    Ok(())
}

pub fn get_table_name(index_id: IndexId) -> String {
    format!("chunks_{index_id}")
}

pub fn init_table(index_id: IndexId, param: &Param) -> Result<(), String> {
    let conn_mtx = DB.lock().unwrap();
    let conn = conn_mtx.as_ref().ok_or("db not initialized")?;
    let table_name = get_table_name(index_id);

    {
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (,
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            index_id INTEGER NOT NULL,
            indexer_id INTEGER NOT NULL,
            file_id INTEGER NOT NULL,
            uri TEXT NOT NULL,
            content_type TEXT NOT NULL,
            content TEXT,
            content_vector BLOB,     -- For sqlite-vector
            metadata BLOB NOT NULL,  -- JSONB
            created_at_utc INTEGER NOT NULL,
            updated_at_utc INTEGER NOT NULL)"
        );
        conn.execute(&stmt, []).map_err(|err| err.to_string())?;
    }

    // init vector
    {
        let dim = param.dimensions;
        let stmt = format!("SELECT vector_init('{table_name}', 'content_vector', 'type=FLOAT32,dimension={dim},distance=COSINE,normalized=1')");
        conn.execute(&stmt, []).map_err(|err| err.to_string())?;
    }

    Ok(())
}

pub fn delete(index_id: IndexId, fild_id: FileId) -> Result<(), String> {
    let conn_mtx = DB.lock().unwrap();
    let conn = conn_mtx.as_ref().ok_or("db not initialized")?;
    let table_name = get_table_name(index_id);

    let stmt = format!("DELETE FROM {table_name} WHERE file_id = ?");
    conn.execute(&stmt, [fild_id])
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub fn clear(index_id: IndexId) -> Result<(), String> {
    let conn_mtx = DB.lock().unwrap();
    let conn = conn_mtx.as_ref().ok_or("db not initialized")?;
    let table_name = get_table_name(index_id);

    let stmt = format!("DELETE FROM {table_name} WHERE index_id = ?");
    conn.execute(&stmt, [index_id])
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub fn insert(index_id: IndexId, chunk: Chunk) -> Result<(), String> {
    let conn_mtx = DB.lock().unwrap();
    let conn = conn_mtx.as_ref().ok_or("db not initialized")?;
    let table_name = get_table_name(index_id);

    let indexer_id = chunk.indexer_id;
    let file_id = chunk.file_id;
    let uri = chunk.uri;
    let content_type = chunk.content_type;
    let mut content_str: String = "NULL".into();
    if let Some(c) = chunk.content {
        content_str = format!("'{c}'");
    }
    let mut content_vec_str = "NULL".into();
    if let Some(c) = chunk.content_vector {
        let ser = serde_json::to_string(&c).map_err(|err| err.to_string())?;
        content_vec_str = format!("vector_as_f32('{ser}')");
    }
    let metadata_str = serde_json::to_string(&chunk.metadata).map_err(|err| err.to_string())?;
    let created_at_utc = chunk.created_at_utc;
    let updated_at_utc = chunk.updated_at_utc;

    let update_column = "index_id, indexer_id, file_id, uri, content_type, content, content_vector, metadata, created_at_utc, updated_at_utc";
    let update_values =
        format!("{index_id}, {indexer_id}, {file_id}, '{uri}', '{content_type}', {content_str}, {content_vec_str}, {metadata_str}, {created_at_utc}, {updated_at_utc}");

    let stmt = format!("INSERT INTO {table_name} ({update_column}) VALUES ({update_values})");
    conn.execute(&stmt, [index_id])
        .map_err(|err| err.to_string())?;

    Ok(())
}

pub fn search(
    index_id: IndexId,
    vec: &Vec<f32>,
    additional_cond_sql: &str,
    offset: i32,
    limit: i32,
) -> Result<Vec<(Chunk, f32)>, String> {
    let conn_mtx = DB.lock().unwrap();
    let conn = conn_mtx.as_ref().ok_or("db not initialized")?;
    let table_name = get_table_name(index_id);

    let vec_ser = serde_json::to_string(vec).map_err(|err| err.to_string())?;

    let stmt = format!(
        "SELECT e.id, e.index_id, e.indexer_id, e.file_id, e.uri, e.content_type, e.content, json(e.metadata), e.created_at_utc, e.updated_at_utc, v.distance FROM {table_name} AS e
        JOIN vector_quantize_scan('{table_name}', 'content_vector', vector_as_f32('{vec_ser}')) AS v
        ON e.id = v.rowid {additional_cond_sql} LIMIT {limit} OFFSET {offset};"
    );
    let mut prepare = conn.prepare(&stmt).map_err(|err| err.to_string())?;
    let iter = prepare
        .query_map([], |row| {
            let metadata_str: String = row.get(7)?;
            let metadata = serde_json::Value::from_str(&metadata_str).unwrap();
            let chunk = Chunk {
                id: row.get(0)?,
                index_id: row.get(1)?,
                indexer_id: row.get(2)?,
                file_id: row.get(3)?,
                uri: row.get(4)?,
                content_type: row.get(5)?,
                content: row.get(6)?,
                content_vector: None,
                metadata: metadata,
                created_at_utc: row.get(8)?,
                updated_at_utc: row.get(9)?,
            };
            let distance: f32 = row.get(10)?;
            let data = (chunk, distance);
            Ok(data)
        })
        .map_err(|err| err.to_string())?;
    let mut ret: Vec<(Chunk, f32)> = Vec::<(Chunk, f32)>::with_capacity(limit as usize);
    for result in iter {
        ret.push(result.map_err(|err| err.to_string())?);
    }
    Ok(ret)
}
