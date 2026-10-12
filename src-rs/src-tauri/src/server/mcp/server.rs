use axum::Router;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct McpServer {
    db: DatabaseConnection,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl McpServer {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDirectoryParams {
    #[schemars(description = "検索対象インデックスのID")]
    pub index_id: String,
    #[schemars(description = "一覧を取得するディレクトリのパス")]
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GrepParams {
    #[schemars(description = "検索対象インデックスのID")]
    pub index_id: String,
    #[schemars(description = "検索するテキスト")]
    pub pattern: String,
    #[schemars(description = "検索対象ファイルまたはディレクトリのパス")]
    pub target: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SemanticSearchParams {
    #[schemars(description = "検索対象インデックスのID")]
    pub index_id: String,
    #[schemars(description = "テキストまたは画像による検索クエリ")]
    pub query: crate::dto::search::SemanticSearchQuery,
    #[schemars(description = "検索対象ファイルまたはディレクトリのパス")]
    pub target: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFileInfoParams {
    #[schemars(description = "検索対象インデックスのID")]
    pub index_id: String,
    #[schemars(description = "ファイルのURI")]
    pub uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IndexSummary {
    id: Uuid,
    name: String,
    description: String,
}

fn parse_index_id(id: &str) -> Result<Uuid, ErrorData> {
    Uuid::parse_str(id)
        .map_err(|err| ErrorData::invalid_params(format!("invalid index_id '{id}': {err}"), None))
}

fn usecase_error(err: String) -> ErrorData {
    ErrorData::internal_error(err, None)
}

fn serialize_result<T: Serialize>(result: T) -> Result<String, ErrorData> {
    serde_json::to_string(&result).map_err(|err| {
        ErrorData::internal_error(format!("failed to serialize search result: {err}"), None)
    })
}

#[tool_router]
impl McpServer {
    #[tool(description = "参照可能なインデックスのIDと名前を取得します。")]
    async fn list_indexes(&self) -> Result<String, ErrorData> {
        let indexes = crate::usecase::indexes::list_indexes(&self.db)
            .await
            .map_err(usecase_error)?;
        let summaries = indexes
            .into_iter()
            .map(|v| v.index)
            .filter(|index| index.use_mcp_server)
            .map(|index| IndexSummary {
                id: index.id,
                name: index.name,
                description: index.description,
            })
            .collect::<Vec<_>>();
        serialize_result(summaries)
    }

    #[tool(description = "指定したインデックス内のディレクトリ内容を取得します。")]
    async fn list_directory(
        &self,
        Parameters(params): Parameters<ListDirectoryParams>,
    ) -> Result<String, ErrorData> {
        let index_id = parse_index_id(&params.index_id)?;
        let result = crate::usecase::search::list_directory(index_id, &params.path, &self.db)
            .await
            .map_err(usecase_error)?;
        serialize_result(result)
    }

    #[tool(description = "指定したインデックスとパスからテキストを検索します。")]
    async fn grep(&self, Parameters(params): Parameters<GrepParams>) -> Result<String, ErrorData> {
        let index_id = parse_index_id(&params.index_id)?;
        let result =
            crate::usecase::search::grep(index_id, &params.pattern, &params.target, &self.db)
                .await
                .map_err(usecase_error)?;
        serialize_result(result)
    }

    #[tool(description = "指定したインデックスとパスからテキストまたは画像で意味検索します。")]
    async fn semantic_search(
        &self,
        Parameters(params): Parameters<SemanticSearchParams>,
    ) -> Result<String, ErrorData> {
        let index_id = parse_index_id(&params.index_id)?;
        let result = crate::usecase::search::semantic_search(
            index_id,
            params.query,
            &params.target,
            &self.db,
        )
        .await
        .map_err(usecase_error)?;
        serialize_result(result)
    }

    #[tool(description = "URIを指定してインデックス内のファイル情報を取得します。")]
    async fn get_file_info(
        &self,
        Parameters(params): Parameters<GetFileInfoParams>,
    ) -> Result<String, ErrorData> {
        let index_id = parse_index_id(&params.index_id)?;
        let result = crate::usecase::search::get_file_info(index_id, &params.uri, &self.db)
            .await
            .map_err(usecase_error)?;
        serialize_result(result)
    }
}

#[tool_handler(
    name = "anydex",
    instructions = "AnyDex のファイル検索サーバー。参照可能なインデックスの確認、ディレクトリ一覧、テキスト検索、意味検索、ファイル情報取得ができます。"
)]
impl ServerHandler for McpServer {}

pub fn mcp_service(
    db: DatabaseConnection,
) -> StreamableHttpService<McpServer, LocalSessionManager> {
    let config = StreamableHttpServerConfig::default();

    StreamableHttpService::new(
        move || Ok(McpServer::new(db.clone())),
        LocalSessionManager::default().into(),
        config,
    )
}

pub fn get_server<T>(db: DatabaseConnection) -> Router<T>
where
    T: Clone + Send + Sync + 'static,
{
    let mcp_service = mcp_service(db);

    let app = Router::new().nest_service("/mcp", mcp_service);

    return app;
}
