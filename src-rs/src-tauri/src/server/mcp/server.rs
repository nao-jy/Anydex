use axum::Router;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler};
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

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
pub struct GetMountainParams {
    #[schemars(description = "山岳ID")]
    pub id: String,
}

#[tool_router]
impl McpServer {
    #[tool(description = "IDを指定して山岳情報を1件取得します。")]
    async fn get_mountain(
        &self,
        Parameters(_params): Parameters<GetMountainParams>,
    ) -> Result<String, ErrorData> {
        let result: Result<(), String> = Ok(());

        match result {
            Ok(_data) => Ok("".to_string()),
            Err(_) => Err(ErrorData::internal_error(
                "山岳情報を取得中に予期せぬエラーが発生しました。",
                None,
            )),
        }
    }
}

#[tool_handler(
    name = "mountix",
    instructions = "Mountix 日本の山岳データ API。百名山などの山岳情報を検索・取得できます。"
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
