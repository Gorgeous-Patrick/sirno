//! MCP server adapter for Sirno.
//!
//! The adapter exposes grouped Sirno command tools over stdio.
//! Command behavior remains in `core`; this module only converts JSON parameters
//! into typed core requests and converts core DTOs into MCP tool results.

use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::{
    ServerHandler, ServiceExt, schemars, schemars::JsonSchema, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

use crate::core::{
    ArtifactAddRequest, ArtifactRemoveRequest, ArtifactRenameRequest, CoreContext, EntryNewRequest,
    EntryPathRequest, FrostCheckoutRequest, LakeInitRequest, PathSelection, QueryColumn,
    QueryColumns, QueryRequest, RgRequest, StructuralFieldState, StructuralFilter,
    StructuralStateFilter, StructuralTarget, TideResolveRequest, TideSelectionRequest,
};
use crate::{CheckMode, EntryId, StructuralEdgeDirection, TideWorkitem};

/// Sirno MCP server bound to one configured project.
#[derive(Clone, Debug)]
pub struct SirnoMcpServer {
    context: CoreContext,
    tool_router: ToolRouter<Self>,
}

impl SirnoMcpServer {
    /// Create an MCP server around one core command context.
    pub fn new(context: CoreContext) -> Self {
        Self { context, tool_router: Self::tool_router() }
    }
}

/// Run one Sirno MCP server on stdio until the client closes the transport.
pub async fn run_stdio(context: CoreContext) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let service = SirnoMcpServer::new(context).serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SirnoMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Sirno tools for one configured project.")
    }
}

#[tool_router(router = tool_router)]
impl SirnoMcpServer {
    /// Create one Markdown entry.
    #[tool(name = "entry_new")]
    fn entry_new(&self, Parameters(params): Parameters<EntryNewParams>) -> McpToolResult {
        let request = EntryNewRequest {
            id: entry_id(params.id)?,
            name: params.name,
            desc: params.desc,
            structural: params.structural.into_targets()?,
            body: params.body,
        };
        result(self.context.entry_new(request))
    }

    /// Rename one entry id and its Sirno references.
    #[tool(name = "entry_rename")]
    fn entry_rename(&self, Parameters(params): Parameters<EntryRenameParams>) -> McpToolResult {
        result(self.context.entry_rename(entry_id(params.old_id)?, entry_id(params.new_id)?))
    }

    /// Freeze one current Frost entry and make its public file read-only.
    #[tool(name = "entry_freeze")]
    fn entry_freeze(&self, Parameters(params): Parameters<EntryIdParams>) -> McpToolResult {
        result(self.context.entry_freeze(entry_id(params.id)?))
    }

    /// Melt one public Markdown entry and make its file writable.
    #[tool(name = "entry_melt")]
    fn entry_melt(&self, Parameters(params): Parameters<EntryIdParams>) -> McpToolResult {
        result(self.context.entry_melt(entry_id(params.id)?))
    }

    /// Show filesystem paths related to one entry.
    #[tool(name = "entry_path")]
    fn entry_path(&self, Parameters(params): Parameters<EntryPathParams>) -> McpToolResult {
        let selection = path_selection(params.entry, params.artifact, params.frost);
        let request = EntryPathRequest::new(
            entry_id(params.id)?,
            selection,
            params.absolute.unwrap_or(false),
        );
        result(self.context.entry_paths(request))
    }

    /// Query public Markdown entries.
    #[tool(name = "entry_query")]
    fn entry_query(&self, Parameters(params): Parameters<EntryQueryParams>) -> McpToolResult {
        let request = QueryRequest {
            terms: params.terms,
            exact_terms: params.exact_terms,
            has: params.has.into_filters()?,
            is: params.is.into_states()?,
            columns: query_columns(params.columns)?,
        };
        result(self.context.entry_query(request))
    }

    /// Run ripgrep in the configured public Markdown lake.
    #[tool(name = "entry_rg")]
    fn entry_rg(&self, Parameters(params): Parameters<EntryRgParams>) -> McpToolResult {
        result(self.context.entry_rg(RgRequest {
            with_generated_footer: params.with_generated_footer,
            args: params.args,
        }))
    }

    /// Return repository witness blocks for one entry.
    #[tool(name = "entry_witness")]
    fn entry_witness(&self, Parameters(params): Parameters<EntryWitnessParams>) -> McpToolResult {
        result(self.context.entry_witness(entry_id(params.id)?, params.full))
    }

    /// List artifacts owned by one entry.
    #[tool(name = "entry_artifact_list")]
    fn entry_artifact_list(&self, Parameters(params): Parameters<EntryIdParams>) -> McpToolResult {
        result(self.context.entry_artifact_list(entry_id(params.id)?))
    }

    /// Copy a file into one entry's artifact tree.
    #[tool(name = "entry_artifact_add")]
    fn entry_artifact_add(
        &self, Parameters(params): Parameters<ArtifactAddParams>,
    ) -> McpToolResult {
        result(self.context.entry_artifact_add(ArtifactAddRequest {
            id: entry_id(params.id)?,
            source: params.source,
            artifact_path: params.artifact_path,
        }))
    }

    /// Rename one artifact path owned by an entry.
    #[tool(name = "entry_artifact_rename")]
    fn entry_artifact_rename(
        &self, Parameters(params): Parameters<ArtifactRenameParams>,
    ) -> McpToolResult {
        result(self.context.entry_artifact_rename(ArtifactRenameRequest {
            id: entry_id(params.id)?,
            old_path: params.old_path,
            new_path: params.new_path,
        }))
    }

    /// Remove one artifact owned by an entry.
    #[tool(name = "entry_artifact_remove")]
    fn entry_artifact_remove(
        &self, Parameters(params): Parameters<ArtifactRemoveParams>,
    ) -> McpToolResult {
        result(self.context.entry_artifact_remove(ArtifactRemoveRequest {
            id: entry_id(params.id)?,
            artifact_path: params.artifact_path,
        }))
    }

    /// Create a Sirno config and ordinary seed entries.
    #[tool(name = "lake_init")]
    fn lake_init(&self, Parameters(params): Parameters<LakeInitParams>) -> McpToolResult {
        result(self.context.lake_init(LakeInitRequest { lake: params.lake }))
    }

    /// Move the configured public Markdown entry lake.
    #[tool(name = "lake_move")]
    fn lake_move(&self, Parameters(params): Parameters<LakeMoveParams>) -> McpToolResult {
        result(self.context.lake_move(params.lake))
    }

    /// Check current entry structure.
    #[tool(name = "lake_check")]
    fn lake_check(&self, Parameters(params): Parameters<LakeCheckParams>) -> McpToolResult {
        result(self.context.lake_check(params.mode.unwrap_or(McpCheckMode::Review).into()))
    }

    /// Render Markdown links in entry footers.
    #[tool(name = "lake_render")]
    fn lake_render(&self, Parameters(params): Parameters<LakeRenderParams>) -> McpToolResult {
        result(self.context.lake_render(params.dry))
    }

    /// Delete generated Markdown link footers.
    #[tool(name = "lake_render_delete")]
    fn lake_render_delete(&self) -> McpToolResult {
        result(self.context.lake_render_delete())
    }

    /// Show the current Sirno project status.
    #[tool(name = "lake_status")]
    fn lake_status(&self) -> McpToolResult {
        result(self.context.lake_status())
    }

    /// Configure Sirno Frost.
    #[tool(name = "frost_init")]
    fn frost_init(&self, Parameters(params): Parameters<FrostInitParams>) -> McpToolResult {
        result(self.context.frost_init(params.frost))
    }

    /// Move the configured Sirno Frost path.
    #[tool(name = "frost_move")]
    fn frost_move(&self, Parameters(params): Parameters<FrostMoveParams>) -> McpToolResult {
        result(self.context.frost_move(params.frost))
    }

    /// Freeze the current public Markdown lake.
    #[tool(name = "frost_commit")]
    fn frost_commit(&self, Parameters(params): Parameters<FrostCommitParams>) -> McpToolResult {
        result(self.context.frost_commit(params.unsafe_resolve_all))
    }

    /// Check out Frost entries into the public Markdown lake.
    #[tool(name = "frost_checkout")]
    fn frost_checkout(&self, Parameters(params): Parameters<FrostCheckoutParams>) -> McpToolResult {
        result(self.context.frost_checkout(FrostCheckoutRequest {
            version: params.version,
            latest: params.latest,
            unsafe_mutable: params.unsafe_mutable,
        }))
    }

    /// Check out the latest Frost version as the mutable current lake.
    #[tool(name = "frost_defrost")]
    fn frost_defrost(&self) -> McpToolResult {
        result(self.context.frost_defrost())
    }

    /// Show tide workitems.
    #[tool(name = "tide_status")]
    fn tide_status(&self, Parameters(params): Parameters<TideStatusParams>) -> McpToolResult {
        result(self.context.tide_status(params.all))
    }

    /// Resolve tide workitems.
    #[tool(name = "tide_resolve")]
    fn tide_resolve(&self, Parameters(params): Parameters<TideResolveParams>) -> McpToolResult {
        result(self.context.tide_resolve(params.into_request()?))
    }

    /// Remove resolved marks from tide workitems.
    #[tool(name = "tide_unresolve")]
    fn tide_unresolve(&self, Parameters(params): Parameters<TideSelectionParams>) -> McpToolResult {
        result(self.context.tide_unresolve(params.into_request()?))
    }

    /// Clear all tide resolutions from the lock.
    #[tool(name = "tide_reset")]
    fn tide_reset(&self) -> McpToolResult {
        result(self.context.tide_reset())
    }
}

type McpToolResult = Result<CallToolResult, String>;

fn result<T: Serialize>(result: Result<T, impl ToString>) -> McpToolResult {
    result.map_err(|error| error.to_string()).and_then(structured_result)
}

fn structured_result<T: Serialize>(value: T) -> McpToolResult {
    let structured = serde_json::to_value(&value).map_err(|error| error.to_string())?;
    let text = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    let mut result = CallToolResult::structured(structured);
    result.content = vec![Content::text(text)];
    Ok(result)
}

fn entry_id(raw: String) -> Result<EntryId, String> {
    EntryId::new(raw).map_err(|error| error.to_string())
}

fn path_selection(
    entry: Option<bool>, artifact: Option<bool>, frost: Option<bool>,
) -> PathSelection {
    let entry = entry.unwrap_or(false);
    let artifact = artifact.unwrap_or(false);
    let frost = frost.unwrap_or(false);
    if !entry && !artifact && !frost {
        PathSelection::all()
    } else {
        PathSelection::new(entry, artifact, frost)
    }
}

fn query_columns(columns: Option<Vec<String>>) -> Result<QueryColumns, String> {
    let Some(columns) = columns else {
        return Ok(QueryColumns::default());
    };
    let columns = columns
        .into_iter()
        .map(|column| QueryColumn::from_str(&column).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(QueryColumns::new(columns))
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryIdParams {
    id: String,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct McpStructuralTargets(Vec<McpStructuralTarget>);

impl McpStructuralTargets {
    fn into_targets(self) -> Result<Vec<StructuralTarget>, String> {
        self.0
            .into_iter()
            .map(|target| {
                Ok(StructuralTarget { field: target.field, target: entry_id(target.target)? })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct McpStructuralTarget {
    field: String,
    target: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryNewParams {
    id: String,
    name: Option<String>,
    desc: String,
    #[serde(default)]
    structural: McpStructuralTargets,
    body: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryRenameParams {
    old_id: String,
    new_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct EntryPathParams {
    id: String,
    entry: Option<bool>,
    artifact: Option<bool>,
    frost: Option<bool>,
    absolute: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct McpStructuralFilters(Vec<McpStructuralFilter>);

impl McpStructuralFilters {
    fn into_filters(self) -> Result<Vec<StructuralFilter>, String> {
        self.0
            .into_iter()
            .map(|filter| {
                Ok(StructuralFilter {
                    field: filter.field,
                    targets: filter
                        .targets
                        .into_iter()
                        .map(entry_id)
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct McpStructuralFilter {
    field: String,
    targets: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct McpStructuralStates(Vec<McpStructuralState>);

impl McpStructuralStates {
    fn into_states(self) -> Result<Vec<StructuralStateFilter>, String> {
        self.0
            .into_iter()
            .map(|state| {
                Ok(StructuralStateFilter { field: state.field, state: state.state.into() })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct McpStructuralState {
    field: String,
    state: McpStructuralFieldState,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum McpStructuralFieldState {
    Present,
    Empty,
    Missing,
}

impl From<McpStructuralFieldState> for StructuralFieldState {
    fn from(value: McpStructuralFieldState) -> Self {
        match value {
            | McpStructuralFieldState::Present => Self::Present,
            | McpStructuralFieldState::Empty => Self::Empty,
            | McpStructuralFieldState::Missing => Self::Missing,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct EntryQueryParams {
    #[serde(default)]
    terms: Vec<String>,
    #[serde(default)]
    exact_terms: Vec<String>,
    #[serde(default)]
    has: McpStructuralFilters,
    #[serde(default)]
    is: McpStructuralStates,
    columns: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct EntryRgParams {
    args: Vec<String>,
    #[serde(default)]
    with_generated_footer: bool,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct EntryWitnessParams {
    id: String,
    #[serde(default)]
    full: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct ArtifactAddParams {
    id: String,
    source: PathBuf,
    artifact_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct ArtifactRenameParams {
    id: String,
    old_path: PathBuf,
    new_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct ArtifactRemoveParams {
    id: String,
    artifact_path: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct LakeInitParams {
    lake: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct LakeMoveParams {
    lake: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct LakeCheckParams {
    mode: Option<McpCheckMode>,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum McpCheckMode {
    Edit,
    Review,
}

impl From<McpCheckMode> for CheckMode {
    fn from(value: McpCheckMode) -> Self {
        match value {
            | McpCheckMode::Edit => Self::Edit,
            | McpCheckMode::Review => Self::Review,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct LakeRenderParams {
    #[serde(default)]
    dry: bool,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct FrostInitParams {
    frost: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct FrostMoveParams {
    frost: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct FrostCommitParams {
    #[serde(default)]
    unsafe_resolve_all: bool,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct FrostCheckoutParams {
    version: Option<u64>,
    #[serde(default)]
    latest: bool,
    #[serde(default)]
    unsafe_mutable: bool,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct TideStatusParams {
    #[serde(default)]
    all: bool,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct TideSelectionParams {
    #[serde(default)]
    neighbors: Vec<String>,
    #[serde(default)]
    workitems: Vec<McpTideWorkitem>,
}

impl TideSelectionParams {
    fn into_request(self) -> Result<TideSelectionRequest, String> {
        Ok(TideSelectionRequest {
            neighbors: self.neighbors.into_iter().map(entry_id).collect::<Result<Vec<_>, _>>()?,
            workitems: self
                .workitems
                .into_iter()
                .map(TideWorkitem::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct TideResolveParams {
    #[serde(default)]
    infer: bool,
    #[serde(default)]
    neighbors: Vec<String>,
    #[serde(default)]
    workitems: Vec<McpTideWorkitem>,
}

impl TideResolveParams {
    fn into_request(self) -> Result<TideResolveRequest, String> {
        Ok(TideResolveRequest {
            infer: self.infer,
            neighbors: self.neighbors.into_iter().map(entry_id).collect::<Result<Vec<_>, _>>()?,
            workitems: self
                .workitems
                .into_iter()
                .map(TideWorkitem::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct McpTideWorkitem {
    ripple: String,
    field: String,
    direction: String,
    neighbor: String,
}

impl TryFrom<McpTideWorkitem> for TideWorkitem {
    type Error = String;

    fn try_from(value: McpTideWorkitem) -> Result<Self, Self::Error> {
        TideWorkitem::new(
            entry_id(value.ripple)?,
            value.field,
            StructuralEdgeDirection::from_str(&value.direction)
                .map_err(|error| error.to_string())?,
            entry_id(value.neighbor)?,
        )
        .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use rmcp::model::{CallToolRequestParams, ClientInfo};
    use rmcp::{ClientHandler, ServiceExt};
    use serde_json::json;

    use super::*;
    use crate::{CONFIG_FILE_NAME, SirnoConfig};

    // sirno:witness:interfaces:begin
    const EXPECTED_TOOLS: &[&str] = &[
        "entry_artifact_add",
        "entry_artifact_list",
        "entry_artifact_remove",
        "entry_artifact_rename",
        "entry_freeze",
        "entry_melt",
        "entry_new",
        "entry_path",
        "entry_query",
        "entry_rename",
        "entry_rg",
        "entry_witness",
        "frost_checkout",
        "frost_commit",
        "frost_defrost",
        "frost_init",
        "frost_move",
        "lake_check",
        "lake_init",
        "lake_move",
        "lake_render",
        "lake_render_delete",
        "lake_status",
        "tide_reset",
        "tide_resolve",
        "tide_status",
        "tide_unresolve",
    ];
    // sirno:witness:interfaces:end

    fn write_project(root: &Path) -> PathBuf {
        let config_path = root.join(CONFIG_FILE_NAME);
        let docs = root.join("docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
---

Body.
",
        )
        .unwrap();
        config_path
    }

    fn structured(result: &CallToolResult) -> &serde_json::Value {
        result.structured_content.as_ref().expect("tool result has structured content")
    }

    #[test]
    fn tool_router_exposes_grouped_tool_surface() {
        let server = SirnoMcpServer::new(CoreContext::new("Sirno.toml"));
        let names = server
            .tool_router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();

        assert_eq!(names, EXPECTED_TOOLS);
    }

    #[test]
    fn direct_tool_call_returns_structured_content_and_pretty_text() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let server = SirnoMcpServer::new(CoreContext::new(&config_path));

        let init = server
            .lake_init(Parameters(LakeInitParams { lake: Some(PathBuf::from("docs")) }))
            .unwrap();
        let entry = server
            .entry_new(Parameters(EntryNewParams {
                id: "alpha".to_owned(),
                name: None,
                desc: "Alpha entry.".to_owned(),
                structural: McpStructuralTargets::default(),
                body: Some("Body.".to_owned()),
            }))
            .unwrap();
        let text = entry
            .content
            .first()
            .and_then(|content| content.as_text())
            .map(|text| text.text.as_str())
            .unwrap();

        assert_eq!(structured(&init)["ok"], true);
        assert_eq!(structured(&entry)["id"], "alpha");
        assert!(text.contains("\n  \"ok\": true,"));
    }

    #[derive(Clone, Debug, Default)]
    struct DummyClient;

    impl ClientHandler for DummyClient {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    #[tokio::test]
    async fn stdio_smoke_lists_tools_and_calls_lake_status() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = write_project(temp.path());
        let server = SirnoMcpServer::new(CoreContext::new(config_path));
        let (server_transport, client_transport) = tokio::io::duplex(8192);

        let server_handle = tokio::spawn(async move {
            server.serve(server_transport).await.unwrap().waiting().await.unwrap();
        });
        let client = DummyClient.serve(client_transport).await.unwrap();

        let tools = client.peer().list_tools(None).await.unwrap();
        assert_eq!(tools.tools.len(), EXPECTED_TOOLS.len());
        assert!(tools.tools.iter().any(|tool| tool.name == "lake_status"));

        let result = client
            .peer()
            .call_tool(
                CallToolRequestParams::new("lake_status")
                    .with_arguments(json!({}).as_object().unwrap().clone()),
            )
            .await
            .unwrap();

        assert_eq!(result.structured_content.as_ref().unwrap()["ok"], true);
        assert_eq!(result.structured_content.as_ref().unwrap()["entry_count"], 1);

        client.cancel().await.unwrap();
        server_handle.await.unwrap();
    }
}
