use crate::bridge::BridgeState;
use crate::types::FindingInput;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{
    Annotated, ListResourceTemplatesResult, PaginatedRequestParams, RawResourceTemplate,
    ReadResourceRequestParams, ReadResourceResult, ResourceContents, ServerCapabilities,
    ServerInfo,
};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::{schemars, tool, tool_handler, tool_router, Json, ServerHandler, ServiceExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub type McpServerResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PostFindingsParams {
    pub source: String,
    pub project: String,
    pub findings: Vec<FindingInput>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ClearFindingsParams {
    pub project: String,
    pub source: String,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct RestoreFindingsParams {
    pub project: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct PostFindingsResult {
    pub added: i64,
    pub affected_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct ClearFindingsResult {
    pub cleared_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct RestoreFindingsResult {
    pub restored_files: Vec<String>,
}

#[derive(Clone)]
pub struct McpServer {
    bridge: BridgeState,
    tool_router: ToolRouter<Self>,
}

impl McpServer {
    pub fn new(bridge: BridgeState) -> Self {
        Self {
            bridge,
            tool_router: Self::tool_router(),
        }
    }

    pub async fn post_findings_direct(&self, params: PostFindingsParams) -> PostFindingsResult {
        let project = self.resolve_project(&params.project).await;
        let added = i64::try_from(params.findings.len()).unwrap_or(i64::MAX);
        let mut store = self.bridge.store.write().await;
        let affected_files = store.post_findings(project.clone(), params.source, params.findings);
        drop(store);
        info!(
            ?project,
            added,
            affected_files = affected_files.len(),
            "posted MCP findings"
        );
        self.bridge.notify_change(project, affected_files.clone());
        PostFindingsResult {
            added,
            affected_files,
        }
    }

    pub async fn clear_findings_direct(&self, params: ClearFindingsParams) -> ClearFindingsResult {
        let project = self.resolve_project(&params.project).await;
        let mut store = self.bridge.store.write().await;
        let cleared_files = store.clear_findings(&project, &params.source);
        drop(store);
        info!(
            ?project,
            cleared_files = cleared_files.len(),
            "cleared MCP findings"
        );
        self.bridge.notify_change(project, cleared_files.clone());
        ClearFindingsResult { cleared_files }
    }

    pub async fn restore_findings_direct(
        &self,
        params: RestoreFindingsParams,
    ) -> RestoreFindingsResult {
        let project = self.resolve_project(&params.project).await;
        let mut store = self.bridge.store.write().await;
        let restored_files = store.restore_findings(&project);
        drop(store);
        info!(
            ?project,
            restored_files = restored_files.len(),
            "restored MCP findings"
        );
        self.bridge.notify_change(project, restored_files.clone());
        RestoreFindingsResult { restored_files }
    }

    async fn resolve_project(&self, project: &str) -> PathBuf {
        let project = PathBuf::from(project);
        if project.is_absolute() {
            return project;
        }

        if let Some(root) = self.bridge.project_root().await {
            info!(
                ?project,
                ?root,
                "mapping relative MCP project to active LSP root"
            );
            return root;
        }

        project
    }
}

#[tool_router]
impl McpServer {
    #[tool(description = "Append code findings to the store")]
    async fn post_findings(
        &self,
        Parameters(params): Parameters<PostFindingsParams>,
    ) -> Json<PostFindingsResult> {
        Json(self.post_findings_direct(params).await)
    }

    #[tool(description = "Clear all findings for a project/source pair")]
    async fn clear_findings(
        &self,
        Parameters(params): Parameters<ClearFindingsParams>,
    ) -> Json<ClearFindingsResult> {
        Json(self.clear_findings_direct(params).await)
    }

    #[tool(description = "Undismiss all findings for a project")]
    async fn restore_findings(
        &self,
        Parameters(params): Parameters<RestoreFindingsParams>,
    ) -> Json<RestoreFindingsResult> {
        Json(self.restore_findings_direct(params).await)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions("Bridge code findings into editor diagnostics")
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::ErrorData> {
        let uri = request.uri;
        let Some(encoded_project) = uri.strip_prefix("findings://") else {
            return Err(rmcp::ErrorData::invalid_params(
                "unsupported resource uri",
                None,
            ));
        };
        let project = PathBuf::from(
            urlencoding::decode(encoded_project)
                .map_err(|error| {
                    rmcp::ErrorData::invalid_params(format!("invalid project path: {error}"), None)
                })?
                .into_owned(),
        );

        let store = self.bridge.store.read().await;
        let findings = store.get_project_findings(&project);
        let json = serde_json::to_string_pretty(&findings).map_err(|error| {
            rmcp::ErrorData::internal_error(format!("failed to serialize findings: {error}"), None)
        })?;

        Ok(ReadResourceResult::new(vec![ResourceContents::text(
            json, uri,
        )
        .with_mime_type("application/json")]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourceTemplatesResult, rmcp::ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            Annotated::new(
                RawResourceTemplate::new("findings://{project_path}", "project findings")
                    .with_description("All findings for a project path as JSON")
                    .with_mime_type("application/json"),
                None,
            ),
        ]))
    }
}

pub async fn serve_stdio(bridge: BridgeState) -> McpServerResult<()> {
    let service = McpServer::new(bridge)
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await?;
    service.waiting().await?;
    Ok(())
}

pub async fn serve_http(
    bridge: BridgeState,
    port: u16,
    cancellation: CancellationToken,
) -> McpServerResult<()> {
    let config =
        StreamableHttpServerConfig::default().with_cancellation_token(cancellation.child_token());
    let service: StreamableHttpService<McpServer, LocalSessionManager> = StreamableHttpService::new(
        move || Ok(McpServer::new(bridge.clone())),
        Default::default(),
        config,
    );
    let router = axum::Router::new().nest_service("/mcp", service);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { cancellation.cancelled_owned().await })
        .await?;
    Ok(())
}
