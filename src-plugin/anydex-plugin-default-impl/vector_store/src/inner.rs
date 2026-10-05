use crate::param::Param;
use anydex_plugin_api_lib::vector_store::params::*;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use anydex_plugin_api_lib::common::Handler;
use anydex_plugin_api_lib::vector_store::interface::{IndexId, SearchHit};
use schemars::schema_for;

use crate::sqlite::{self};

struct HandleInfo {
    pub param: Param,
    pub index_id: IndexId,
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
    let schema = serde_json::from_value::<Param>(args.params).map_err(|err| err.to_string())?;
    let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
    let handler = (*handler_mtx).clone();
    (*handler_mtx) += 1;

    let info = HandleInfo {
        index_id,
        param: schema,
    };
    sqlite::init_table(index_id, &info.param)?;
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
    match args.condition {
        DeleteCond::ByFileId(file_id) => sqlite::delete_file(info.index_id, file_id)?,
        DeleteCond::ByIndexerId(indexer_id) => sqlite::delete_indexer(info.index_id, indexer_id)?,
    };
    Ok(DeleteChunksData {})
}

pub fn clear_chunks(args: ClearChunksArgs) -> Result<ClearChunksData, String> {
    let handler = args.handler;
    let info_mtx = HANDLERS.lock().unwrap();
    let info = (*info_mtx)
        .get(&handler)
        .ok_or(format!("handler {handler} not found"))?;
    sqlite::clear(info.index_id)?;
    Ok(ClearChunksData {})
}

pub fn insert_chunks(args: InsertChunksArgs) -> Result<InsertChunksData, String> {
    let handler = args.handler;
    let info_mtx = HANDLERS.lock().unwrap();
    let info = (*info_mtx)
        .get(&handler)
        .ok_or(format!("handler {handler} not found"))?;
    for chunk in args.chunks {
        sqlite::insert(info.index_id, chunk)?;
    }
    Ok(InsertChunksData {})
}

pub fn search(args: SearchArgs) -> Result<SearchData, String> {
    let handler = args.handler;
    let info_mtx = HANDLERS.lock().unwrap();
    let info = (*info_mtx)
        .get(&handler)
        .ok_or(format!("handler {handler} not found"))?;

    let vec: Vec<f32> = args.search_request.embedding;
    let cond = args.search_request.filter_condition;
    let result = sqlite::search(
        info.index_id,
        &vec,
        &cond,
        args.search_request.offset,
        args.search_request.limit,
    )?;
    Ok(SearchData {
        results: result
            .iter()
            .map(|(chunk, score)| SearchHit {
                chunk: chunk.to_owned(),
                score: score.to_owned(),
            })
            .collect(),
    })
}
