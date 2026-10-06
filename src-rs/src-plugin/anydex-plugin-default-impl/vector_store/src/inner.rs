use crate::param::Param;
use anydex_plugin_api_lib::vector_store::params::*;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::repository::DB;
use anydex_plugin_api_lib::common::Handler;
use schemars::schema_for;

struct HandleInfo {
    pub db: DB,
}

static HANDLERS: LazyLock<Mutex<HashMap<Handler, HandleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLER_COUNTER: Mutex<u64> = Mutex::new(0);

pub fn get_param_schema(_args: GetParamSchemaArgs) -> Result<GetParamSchemaData, String> {
    let schema = schema_for!(Param);
    return Ok(GetParamSchemaData {
        schema: serde_json::to_value(&schema).map_err(|err| err.to_string())?,
    });
}

pub fn get_handler(args: GetHandlerArgs) -> Result<GetHandlerData, String> {
    let index_id = args.index_id;
    let param = serde_json::from_value::<Param>(args.params).map_err(|err| err.to_string())?;
    let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
    let handler = (*handler_mtx).clone();
    (*handler_mtx) += 1;

    let params = param.clone();
    let db = DB::init_table(index_id, &params)?;

    let info = HandleInfo { db: db };
    HANDLERS.lock().unwrap().insert(handler, info);
    Ok(GetHandlerData { handler })
}

pub fn free_handler(args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
    HANDLERS.lock().unwrap().remove(&args.handler);
    Ok(FreeHandlerData {})
}

pub fn delete_chunks(args: DeleteChunksArgs) -> Result<DeleteChunksData, String> {
    let handler = args.handler;
    let info_mtx = HANDLERS.lock().unwrap();
    let info = (*info_mtx)
        .get(&handler)
        .ok_or(format!("handler {handler} not found"))?;
    let db = &info.db;
    match args.condition {
        DeleteCond::ByFileId(file_id) => db.delete_by_file(file_id)?,
        DeleteCond::ByIndexerId(indexer_id) => db.delete_by_indexer(indexer_id)?,
    };
    Ok(DeleteChunksData {})
}

pub fn clear_chunks(args: ClearChunksArgs) -> Result<ClearChunksData, String> {
    let handler = args.handler;
    let info_mtx = HANDLERS.lock().unwrap();
    let info = (*info_mtx)
        .get(&handler)
        .ok_or(format!("handler {handler} not found"))?;
    let db = &info.db;
    db.clear()?;
    Ok(ClearChunksData {})
}

pub fn insert_chunks(args: InsertChunksArgs) -> Result<InsertChunksData, String> {
    let handler = args.handler;
    let info_mtx = HANDLERS.lock().unwrap();
    let info = (*info_mtx)
        .get(&handler)
        .ok_or(format!("handler {handler} not found"))?;
    let db = &info.db;
    db.insert(args.chunks)?;
    Ok(InsertChunksData {})
}

pub fn search(args: SearchArgs) -> Result<SearchData, String> {
    let handler = args.handler;
    let info_mtx = HANDLERS.lock().unwrap();
    let info = (*info_mtx)
        .get(&handler)
        .ok_or(format!("handler {handler} not found"))?;
    let db = &info.db;

    let vec: Vec<f32> = args.search_request.embedding;
    let cond = args.search_request.filter_condition;
    let result = db.search(
        &vec,
        &cond,
        args.search_request.offset,
        args.search_request.limit,
    )?;
    Ok(SearchData { results: result })
}
