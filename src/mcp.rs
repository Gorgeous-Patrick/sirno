//! MCP server adapter for Sirno.
//!
//! The adapter exposes grouped Sirno command tools and skill resources over stdio.
//! Command behavior remains in `surface`; this module only converts JSON parameters
//! into typed surface requests and converts surface DTOs into MCP tool results.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;
use std::future::{self, Future};
use std::path::PathBuf;
use std::str::FromStr;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    Annotated, CallToolResult, Content, ListResourceTemplatesResult, ListResourcesResult,
    PaginatedRequestParams, RawResource, RawResourceTemplate, ReadResourceRequestParams,
    ReadResourceResult, Resource, ResourceContents, ResourceTemplate, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt, schemars, schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

use crate::surface::{
    ArtifactAddRequest, ArtifactRemoveRequest, ArtifactRenameRequest, EntryNewRequest,
    EntryPathsRequest, EntryReadContent, EntryReadRequest, LakeInitRequest, PathSelection,
    QueryColumn, QueryColumnSelection, QueryColumns, QueryRequest, RgRequest, SkillResourceContext,
    StatusMode, StatusRequest, StructuralFieldState, StructuralFilter, StructuralStateFilter,
    StructuralTarget, SurfaceContext, TideResolveRequest, TideSelectionRequest, TideStatusMode,
    UpstreamAddRequest, UpstreamCrystallizeRequest,
};
use crate::surface::{CommandError, format_command_error};
use crate::{
    CheckMode, EntryAddress, EntryAtom, EntryIntrinsicFields, StructuralEdgeDirection,
    TideWorkitem, UpstreamSettings,
};

const SKILL_RESOURCE_MIME_TYPE: &str = "text/markdown";
const SKILL_PROJECT_CONTEXT_TOKEN: &str = "{{SIRNO_ACTIVE_PROJECT_METADATA}}";
const ENTRY_RESOURCE_MIME_TYPE: &str = "text/markdown";
const ENTRY_RESOURCE_URI_PREFIX: &str = "sirno://entries/";
const ENTRY_RESOURCE_URI_TEMPLATE: &str = "sirno://entries/{id}";

// sirno:witness:design-doc-writer-skill:begin
const DESIGN_DOC_WRITER_SKILL_RESOURCE: SkillResourceSpec = SkillResourceSpec {
    uri: "sirno://skills/design-doc-writer",
    name: "design-doc-writer",
    title: "Design Doc Writer",
    description: "Full design-doc-writer skill text.",
    template: include_str!(
        "../.sirno/lake/.artifacts/design-doc-writer-skill/SKILL.full.template.md"
    ),
};
// sirno:witness:design-doc-writer-skill:end

// sirno:witness:agent-skills:begin
const SKILL_RESOURCES: &[SkillResourceSpec] = &[
    DESIGN_DOC_WRITER_SKILL_RESOURCE,
    SkillResourceSpec {
        uri: "sirno://skills/sirno-editor",
        name: "sirno-editor",
        title: "Sirno Editor",
        description: "Full Sirno editor skill text.",
        template: include_str!(
            "../.sirno/lake/.artifacts/repository-editing-discipline/SKILL.full.template.md"
        ),
    },
    SkillResourceSpec {
        uri: "sirno://skills/sirno-actualizer",
        name: "sirno-actualizer",
        title: "Sirno Actualizer",
        description: "Full Sirno actualizer skill text.",
        template: include_str!(
            "../.sirno/lake/.artifacts/actualization-discipline/SKILL.full.template.md"
        ),
    },
    SkillResourceSpec {
        uri: "sirno://skills/sirno-internalizer",
        name: "sirno-internalizer",
        title: "Sirno Internalizer",
        description: "Full Sirno internalizer skill text.",
        template: include_str!(
            "../.sirno/lake/.artifacts/internalization-discipline/SKILL.full.template.md"
        ),
    },
    SkillResourceSpec {
        uri: "sirno://skills/sirno-narrative-session",
        name: "sirno-narrative-session",
        title: "Sirno Narrative Session",
        description: "Full Sirno narrative-session skill text.",
        template: include_str!(
            "../.sirno/lake/.artifacts/narrative-session-discipline/SKILL.full.template.md"
        ),
    },
    SkillResourceSpec {
        uri: "sirno://skills/sirno-skill-synthesizer",
        name: "sirno-skill-synthesizer",
        title: "Sirno Skill Synthesizer",
        description: "Full Sirno skill-synthesizer text.",
        template: include_str!(
            "../.sirno/lake/.artifacts/skill-synthesis-discipline/SKILL.full.template.md"
        ),
    },
    SkillResourceSpec {
        uri: "sirno://skills/sirno-curator",
        name: "sirno-curator",
        title: "Sirno Curator",
        description: "Full Sirno curator skill text.",
        template: include_str!(
            "../.sirno/lake/.artifacts/lake-curation-discipline/SKILL.full.template.md"
        ),
    },
    SkillResourceSpec {
        uri: "sirno://skills/sirno-finalizer",
        name: "sirno-finalizer",
        title: "Sirno Finalizer",
        description: "Full Sirno finalizer skill text.",
        template: include_str!(
            "../.sirno/lake/.artifacts/finalization-discipline/SKILL.full.template.md"
        ),
    },
];
// sirno:witness:agent-skills:end

#[derive(Clone, Copy, Debug)]
struct SkillResourceSpec {
    uri: &'static str,
    name: &'static str,
    title: &'static str,
    description: &'static str,
    template: &'static str,
}

impl SkillResourceSpec {
    fn for_uri(uri: &str) -> Option<&'static Self> {
        SKILL_RESOURCES.iter().find(|resource| resource.uri == uri)
    }

    fn as_resource(&self) -> Resource {
        Annotated::new(
            RawResource::new(self.uri, self.name)
                .with_title(self.title)
                .with_description(self.description)
                .with_mime_type(SKILL_RESOURCE_MIME_TYPE),
            None,
        )
    }

    fn as_resource_contents(&self, context: &SurfaceContext) -> ResourceContents {
        ResourceContents::text(self.render(context), self.uri)
            .with_mime_type(SKILL_RESOURCE_MIME_TYPE)
    }

    fn render(&self, context: &SurfaceContext) -> String {
        if !self.template.contains(SKILL_PROJECT_CONTEXT_TOKEN) {
            return self.template.to_owned();
        }
        self.template.replace(SKILL_PROJECT_CONTEXT_TOKEN, &render_skill_project_context(context))
    }
}

fn render_skill_project_context(context: &SurfaceContext) -> String {
    match context.skill_resource_context() {
        | Ok(context) => render_available_skill_project_context(&context),
        | Err(error) => format!(
            "## Active Project Metadata\n\n\
             This section is generated by Sirno MCP when the resource is read.\n\
             The active project metadata is unavailable: {error}.\n\
             Bind the server with `sirno_cwd` and run `sirno_lake_check` to refresh \
             `.sirno/meta.toml` before relying on project field names.\n"
        ),
    }
}

fn render_available_skill_project_context(context: &SkillResourceContext) -> String {
    let mut out = String::new();
    out.push_str("## Active Project Metadata\n\n");
    out.push_str("This section is generated by Sirno MCP when the resource is read.\n");
    out.push_str("Use these fields for the active project.\n");
    out.push_str("Do not assume another project has the same intrinsic or structural fields.\n\n");
    writeln!(out, "Config path: {}.", inline_code(&context.config_path)).unwrap();
    writeln!(out, "Lake path: {}.", inline_code(&context.lake_path)).unwrap();
    writeln!(out, "Meta registry: {}.", inline_code(&context.meta_path)).unwrap();
    out.push_str("Default query columns: `id`, `path`.\n\n");

    out.push_str("Discovered intrinsic fields:\n");
    if context.intrinsic_fields.is_empty() {
        out.push_str("- none\n");
    } else {
        for field in &context.intrinsic_fields {
            writeln!(
                out,
                "- {} defined by {}",
                inline_code(&field.field),
                inline_code(field.entry.as_str())
            )
            .unwrap();
        }
    }

    out.push_str("\nDiscovered structural relations:\n");
    if context.structural_fields.is_empty() {
        out.push_str("- none\n");
    } else {
        for field in &context.structural_fields {
            writeln!(
                out,
                "- {} defined by {}",
                inline_code(&field.field),
                inline_code(field.entry.as_str())
            )
            .unwrap();
        }
    }
    out.push_str(
        "\nThese fields come from `.sirno/meta.toml`.\n\
         Read a field's defining entry before relying on that field's semantics.\n",
    );
    out
}

fn inline_code(value: &str) -> String {
    if value.contains('`') { format!("`` {value} ``") } else { format!("`{value}`") }
}

fn entry_resource_template() -> ResourceTemplate {
    Annotated::new(
        RawResourceTemplate::new(ENTRY_RESOURCE_URI_TEMPLATE, "entry")
            .with_title("Sirno Entry")
            .with_description("Full Markdown source for one Sirno Lake entry by id.")
            .with_mime_type(ENTRY_RESOURCE_MIME_TYPE),
        None,
    )
}

/// Sirno MCP server for one config path.
///
/// Relative config paths are resolved when tools read them,
/// so changing the process current working directory can change the active project.
#[derive(Clone, Debug)]
pub struct SirnoMcpServer {
    context: SurfaceContext,
    tool_router: ToolRouter<Self>,
}

impl SirnoMcpServer {
    /// Create an MCP server around one surface command context.
    pub fn new(context: SurfaceContext) -> Self {
        Self { context, tool_router: Self::tool_router() }
    }
}

/// Run one Sirno MCP server on stdio until the client closes the transport.
pub async fn run_stdio(
    context: SurfaceContext,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let service = SirnoMcpServer::new(context).serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SirnoMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_resources().enable_tools().build())
            .with_instructions("Sirno tools for the project resolved by the active config path.")
    }

    // sirno:witness:mcp-interface:begin
    fn list_resources(
        &self, _request: Option<PaginatedRequestParams>, _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + MaybeSendFuture + '_ {
        let resources =
            SKILL_RESOURCES.iter().map(SkillResourceSpec::as_resource).collect::<Vec<_>>();
        future::ready(Ok(ListResourcesResult::with_all_items(resources)))
    }

    fn list_resource_templates(
        &self, _request: Option<PaginatedRequestParams>, _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, McpError>> + MaybeSendFuture + '_
    {
        future::ready(Ok(ListResourceTemplatesResult::with_all_items(vec![
            entry_resource_template(),
        ])))
    }

    fn read_resource(
        &self, request: ReadResourceRequestParams, _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + MaybeSendFuture + '_ {
        let result = if let Some(resource) = SkillResourceSpec::for_uri(&request.uri) {
            Ok(ReadResourceResult::new(vec![resource.as_resource_contents(&self.context)]))
        } else if let Some(raw_id) = request.uri.strip_prefix(ENTRY_RESOURCE_URI_PREFIX) {
            entry_address(raw_id.to_owned())
                .map_err(|error| McpError::invalid_params(error, None))
                .and_then(|id| {
                    self.context
                        .entry_read(EntryReadRequest::new(id, EntryReadContent::Source))
                        .map_err(|error| McpError::resource_not_found(error.to_string(), None))
                })
                .and_then(|entry| {
                    let Some(source) = entry.source else {
                        return Err(McpError::resource_not_found(
                            format!("entry source was not returned for {}", entry.id),
                            None,
                        ));
                    };
                    Ok(ReadResourceResult::new(vec![
                        ResourceContents::text(source, request.uri)
                            .with_mime_type(ENTRY_RESOURCE_MIME_TYPE),
                    ]))
                })
        } else {
            Err(McpError::resource_not_found(format!("resource not found: {}", request.uri), None))
        };
        future::ready(result)
    }
    // sirno:witness:mcp-interface:end
}

#[tool_router(router = tool_router)]
impl SirnoMcpServer {
    /// Read or change the server process current working directory.
    #[tool(name = "sirno_cwd")]
    fn cwd(&self, Parameters(params): Parameters<CwdParams>) -> McpToolResult {
        result(self.context.cwd(params.path))
    }

    /// Create one Markdown entry.
    // sirno:witness:mcp-interface:begin
    #[tool(name = "sirno_entry_new")]
    fn entry_new(&self, Parameters(params): Parameters<EntryNewParams>) -> McpToolResult {
        let request = EntryNewRequest {
            id: entry_address(params.id)?,
            intrinsic: entry_intrinsic_fields(params.intrinsic),
            structural: params.structural.into_targets()?,
            body: params.body,
        };
        result(self.context.entry_new(request))
    }
    // sirno:witness:mcp-interface:end

    /// Rename one entry address and its Sirno references.
    #[tool(name = "sirno_entry_rename")]
    fn entry_rename(&self, Parameters(params): Parameters<EntryRenameParams>) -> McpToolResult {
        result(
            self.context.entry_rename(entry_address(params.old_id)?, entry_address(params.new_id)?),
        )
    }

    /// Freeze one current lake entry and make its file read-only.
    #[tool(name = "sirno_entry_freeze")]
    fn entry_freeze(
        &self, Parameters(params): Parameters<EntryAddressOnlyParams>,
    ) -> McpToolResult {
        result(self.context.entry_freeze(entry_address(params.id)?))
    }

    /// Melt one Sirno Lake Markdown entry and make its file writable.
    #[tool(name = "sirno_entry_melt")]
    fn entry_melt(&self, Parameters(params): Parameters<EntryAddressOnlyParams>) -> McpToolResult {
        result(self.context.entry_melt(entry_address(params.id)?))
    }

    /// Show filesystem paths related to one entry.
    #[tool(name = "sirno_entry_path")]
    fn entry_paths(&self, Parameters(params): Parameters<EntryPathsParams>) -> McpToolResult {
        let selection = path_selection(params.entry, params.artifact);
        let request = EntryPathsRequest::new(
            entry_address(params.id)?,
            selection,
            params.absolute.unwrap_or(false),
        );
        result(self.context.entry_paths(request))
    }

    /// Read one Sirno Lake Markdown entry.
    #[tool(name = "sirno_entry_read")]
    fn entry_read(&self, Parameters(params): Parameters<EntryReadParams>) -> McpToolResult {
        result(
            self.context.entry_read(EntryReadRequest::new(
                entry_address(params.id)?,
                params.content.into(),
            )),
        )
    }

    /// Query Sirno Lake Markdown entries.
    #[tool(name = "sirno_entry_query")]
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

    /// Run ripgrep in the configured Sirno Lake.
    #[tool(name = "sirno_entry_rg")]
    fn entry_rg(&self, Parameters(params): Parameters<EntryRgParams>) -> McpToolResult {
        result(self.context.entry_rg(RgRequest {
            with_generated_footer: params.with_generated_footer,
            args: params.args,
        }))
    }

    /// Return repository witness blocks for one entry.
    #[tool(name = "sirno_entry_witness")]
    fn entry_witness(&self, Parameters(params): Parameters<EntryWitnessParams>) -> McpToolResult {
        result(self.context.entry_witness(entry_address(params.id)?, params.verbose_json))
    }

    /// List artifacts owned by one entry.
    #[tool(name = "sirno_entry_artifact_list")]
    fn entry_artifact_list(
        &self, Parameters(params): Parameters<EntryAddressOnlyParams>,
    ) -> McpToolResult {
        result(self.context.entry_artifact_list(entry_address(params.id)?))
    }

    /// Copy a file into one entry's artifact tree.
    #[tool(name = "sirno_entry_artifact_add")]
    fn entry_artifact_add(
        &self, Parameters(params): Parameters<ArtifactAddParams>,
    ) -> McpToolResult {
        result(self.context.entry_artifact_add(ArtifactAddRequest {
            id: entry_address(params.id)?,
            source: params.source,
            artifact_path: params.artifact_path,
        }))
    }

    /// Rename one artifact path owned by an entry.
    #[tool(name = "sirno_entry_artifact_rename")]
    fn entry_artifact_rename(
        &self, Parameters(params): Parameters<ArtifactRenameParams>,
    ) -> McpToolResult {
        result(self.context.entry_artifact_rename(ArtifactRenameRequest {
            id: entry_address(params.id)?,
            old_path: params.old_path,
            new_path: params.new_path,
        }))
    }

    /// Remove one artifact owned by an entry.
    #[tool(name = "sirno_entry_artifact_remove")]
    fn entry_artifact_remove(
        &self, Parameters(params): Parameters<ArtifactRemoveParams>,
    ) -> McpToolResult {
        result(self.context.entry_artifact_remove(ArtifactRemoveRequest {
            id: entry_address(params.id)?,
            artifact_path: params.artifact_path,
        }))
    }

    /// Create a Sirno config and ordinary seed entries.
    #[tool(name = "sirno_lake_init")]
    // sirno:witness:lake-commands:begin
    fn lake_init(&self, Parameters(params): Parameters<LakeInitParams>) -> McpToolResult {
        result(self.context.lake_init(LakeInitRequest { lake: params.lake }))
    }
    // sirno:witness:lake-commands:end

    /// Move the configured Sirno Lake.
    #[tool(name = "sirno_lake_move")]
    // sirno:witness:lake-commands:begin
    fn lake_move(&self, Parameters(params): Parameters<LakeMoveParams>) -> McpToolResult {
        result(self.context.lake_move(params.lake))
    }
    // sirno:witness:lake-commands:end

    /// Check current entry structure.
    #[tool(name = "sirno_lake_check")]
    // sirno:witness:lake-commands:begin
    fn lake_check(&self, Parameters(params): Parameters<LakeCheckParams>) -> McpToolResult {
        result(self.context.lake_check(params.mode.unwrap_or(McpCheckMode::Review).into()))
    }
    // sirno:witness:lake-commands:end

    // sirno:witness:mist-commands:begin
    /// Render Markdown links for one misty lake projection.
    #[tool(name = "sirno_mist_render")]
    fn mist_render(&self, Parameters(params): Parameters<MistRenderParams>) -> McpToolResult {
        result(self.context.mist_render(mist_name(params.mist)?, params.dry))
    }

    /// Show pending mist ripples and stale projection state.
    #[tool(name = "sirno_mist_status")]
    fn mist_status(&self, Parameters(params): Parameters<MistNameParams>) -> McpToolResult {
        result(self.context.mist_status(mist_name(params.mist)?))
    }

    /// Intake edited Markdown entry sources from a misty lake into the reservoir.
    #[tool(name = "sirno_mist_intake")]
    fn mist_intake(&self, Parameters(params): Parameters<MistNameParams>) -> McpToolResult {
        result(self.context.mist_intake(mist_name(params.mist)?))
    }

    /// Delete generated Markdown link footers for one misty lake projection.
    #[tool(name = "sirno_mist_render_delete")]
    fn mist_render_delete(&self, Parameters(params): Parameters<MistNameParams>) -> McpToolResult {
        result(self.context.mist_render_delete(mist_name(params.mist)?))
    }
    // sirno:witness:mist-commands:end

    // sirno:witness:project-status-commands:begin
    /// Show the current Sirno project status.
    #[tool(name = "sirno_status")]
    fn status(&self, Parameters(params): Parameters<StatusParams>) -> McpToolResult {
        result(self.context.status(StatusRequest::new(params.show.into())))
    }
    // sirno:witness:project-status-commands:end

    // sirno:witness:upstream-commands:begin
    /// Add or replace one Git-backed upstream and crystallize it.
    #[tool(name = "sirno_upstream_add")]
    fn upstream_add(&self, Parameters(params): Parameters<UpstreamAddParams>) -> McpToolResult {
        result(self.context.upstream_add(params.into_request()?))
    }

    /// Remove one upstream declaration and its glacier.
    #[tool(name = "sirno_upstream_remove")]
    fn upstream_remove(
        &self, Parameters(params): Parameters<EntryAtomOnlyParams>,
    ) -> McpToolResult {
        result(self.context.upstream_remove(entry_atom(params.domain)?))
    }

    /// Crystallize configured upstream lakes into glaciers.
    #[tool(name = "sirno_upstream_crystallize")]
    fn upstream_crystallize(
        &self, Parameters(params): Parameters<UpstreamCrystallizeParams>,
    ) -> McpToolResult {
        result(self.context.upstream_crystallize(params.into_request()?))
    }

    /// Refresh upstream locks and glaciers.
    #[tool(name = "sirno_upstream_update")]
    fn upstream_update(
        &self, Parameters(params): Parameters<UpstreamDomainsParams>,
    ) -> McpToolResult {
        result(self.context.upstream_update(entry_atoms(params.domains)?))
    }

    /// Show upstream lock and cache status.
    #[tool(name = "sirno_upstream_status")]
    fn upstream_status(&self) -> McpToolResult {
        result(self.context.upstream_status())
    }
    // sirno:witness:upstream-commands:end

    // sirno:witness:anchor-commands:begin
    /// Show lake ripples against the accepted anchor baseline.
    #[tool(name = "sirno_anchor_status")]
    fn anchor_status(&self) -> McpToolResult {
        result(self.context.anchor_status())
    }

    /// Validate the accepted anchor baseline.
    #[tool(name = "sirno_anchor_check")]
    fn anchor_check(&self) -> McpToolResult {
        result(self.context.anchor_check())
    }

    /// Accept the current lake as the new anchor baseline.
    #[tool(name = "sirno_anchor_update")]
    fn anchor_update(&self) -> McpToolResult {
        result(self.context.anchor_update())
    }
    // sirno:witness:anchor-commands:end

    // sirno:witness:tide-commands:begin
    /// Show tide review status.
    #[tool(name = "sirno_tide_status")]
    fn tide_status(&self, Parameters(params): Parameters<TideStatusParams>) -> McpToolResult {
        result(self.context.tide_status(params.show.into()))
    }

    /// Resolve tide workitems.
    #[tool(name = "sirno_tide_resolve")]
    fn tide_resolve(&self, Parameters(params): Parameters<TideResolveParams>) -> McpToolResult {
        result(self.context.tide_resolve(params.into_request()?))
    }

    /// Remove resolved marks from tide workitems.
    #[tool(name = "sirno_tide_unresolve")]
    fn tide_unresolve(&self, Parameters(params): Parameters<TideSelectionParams>) -> McpToolResult {
        result(self.context.tide_unresolve(params.into_request()?))
    }

    /// Clear all tide resolutions from the Tide file.
    #[tool(name = "sirno_tide_reset")]
    fn tide_reset(&self) -> McpToolResult {
        result(self.context.tide_reset())
    }
    // sirno:witness:tide-commands:end
}

type McpToolResult = Result<CallToolResult, String>;

// sirno:witness:mcp-interface:begin
fn result<T: Serialize>(result: Result<T, CommandError>) -> McpToolResult {
    result.map_err(|error| format_command_error(&error)).and_then(structured_result)
}

fn structured_result<T: Serialize>(value: T) -> McpToolResult {
    let structured = serde_json::to_value(&value).map_err(|error| error.to_string())?;
    let text = tool_result_text(&structured);
    let mut result = CallToolResult::structured(structured);
    if let Some(text) = text {
        result.content = vec![Content::text(text)];
    }
    Ok(result)
}

fn tool_result_text(value: &serde_json::Value) -> Option<String> {
    let message = value
        .get("message")
        .and_then(serde_json::Value::as_str)
        .filter(|message| !message.is_empty());
    if let Some(message) = message {
        return Some(message.to_owned());
    }

    value.get("ok").and_then(serde_json::Value::as_bool).map(|ok| format!("ok: {ok}"))
}
// sirno:witness:mcp-interface:end

fn entry_address(raw: String) -> Result<EntryAddress, String> {
    EntryAddress::new(raw).map_err(|error| error.to_string())
}

fn entry_intrinsic_fields(fields: BTreeMap<String, String>) -> EntryIntrinsicFields {
    fields.into_iter().collect()
}

fn entry_atom(raw: String) -> Result<EntryAtom, String> {
    EntryAtom::new(raw).map_err(|error| error.to_string())
}

fn entry_atoms(raw: Vec<String>) -> Result<Vec<EntryAtom>, String> {
    raw.into_iter().map(entry_atom).collect()
}

fn mist_name(raw: Option<String>) -> Result<Option<EntryAtom>, String> {
    raw.map(entry_atom).transpose()
}

fn path_selection(entry: Option<bool>, artifact: Option<bool>) -> PathSelection {
    let entry = entry.unwrap_or(false);
    let artifact = artifact.unwrap_or(false);
    if !entry && !artifact { PathSelection::all() } else { PathSelection::new(entry, artifact) }
}

// sirno:witness:mcp-interface:begin
fn query_columns(columns: Option<Vec<String>>) -> Result<QueryColumnSelection, String> {
    let Some(columns) = columns else {
        return Ok(QueryColumnSelection::Default);
    };
    if columns.is_empty() {
        return Ok(QueryColumnSelection::Options);
    }
    let columns = columns
        .into_iter()
        .map(|column| QueryColumn::from_str(&column).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(QueryColumnSelection::Selected(QueryColumns::new(columns)))
}
// sirno:witness:mcp-interface:end

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct CwdParams {
    path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryAddressOnlyParams {
    id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryReadParams {
    id: String,
    /// Entry content to include. Defaults to parsed body text.
    #[serde(default)]
    content: McpEntryReadContent,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum McpEntryReadContent {
    /// Return entry metadata and paths only.
    Metadata,
    /// Return metadata, paths, and parsed body text.
    #[default]
    Body,
    /// Return metadata, paths, and full stored Markdown source.
    Source,
    /// Return metadata, paths, parsed body text, and full stored Markdown source.
    Full,
}

impl From<McpEntryReadContent> for EntryReadContent {
    fn from(value: McpEntryReadContent) -> Self {
        match value {
            | McpEntryReadContent::Metadata => Self::Metadata,
            | McpEntryReadContent::Body => Self::Body,
            | McpEntryReadContent::Source => Self::Source,
            | McpEntryReadContent::Full => Self::Full,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryAtomOnlyParams {
    /// Glacier domain.
    domain: String,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct McpStructuralTargets(Vec<McpStructuralTarget>);

impl McpStructuralTargets {
    fn into_targets(self) -> Result<Vec<StructuralTarget>, String> {
        self.0
            .into_iter()
            .map(|target| {
                Ok(StructuralTarget { field: target.field, target: entry_address(target.target)? })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct McpStructuralTarget {
    field: String,
    target: String,
}

// sirno:witness:mcp-interface:begin
#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryNewParams {
    id: String,
    intrinsic: BTreeMap<String, String>,
    #[serde(default)]
    structural: McpStructuralTargets,
    body: Option<String>,
}
// sirno:witness:mcp-interface:end

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct EntryRenameParams {
    old_id: String,
    new_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct EntryPathsParams {
    id: String,
    entry: Option<bool>,
    artifact: Option<bool>,
    absolute: Option<bool>,
}

// sirno:witness:mcp-interface:begin
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
enum McpStructuralFilters {
    One(McpStructuralFilterInput),
    Many(Vec<McpStructuralFilterInput>),
}

impl Default for McpStructuralFilters {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl McpStructuralFilters {
    fn into_filters(self) -> Result<Vec<StructuralFilter>, String> {
        self.into_inputs().into_iter().map(McpStructuralFilterInput::into_filter).collect()
    }

    fn into_inputs(self) -> Vec<McpStructuralFilterInput> {
        match self {
            | Self::One(input) => vec![input],
            | Self::Many(inputs) => inputs,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
enum McpStructuralFilterInput {
    Object(McpStructuralFilter),
    Compact(String),
}

impl McpStructuralFilterInput {
    fn into_filter(self) -> Result<StructuralFilter, String> {
        match self {
            | Self::Object(filter) => Ok(StructuralFilter {
                field: filter.field,
                targets: filter
                    .targets
                    .into_iter()
                    .map(entry_address)
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            | Self::Compact(raw) => {
                StructuralFilter::from_str(&raw).map_err(|error| error.to_string())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct McpStructuralFilter {
    field: String,
    targets: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
enum McpStructuralStates {
    One(McpStructuralStateInput),
    Many(Vec<McpStructuralStateInput>),
}

impl Default for McpStructuralStates {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl McpStructuralStates {
    fn into_states(self) -> Result<Vec<StructuralStateFilter>, String> {
        self.into_inputs().into_iter().map(McpStructuralStateInput::into_state).collect()
    }

    fn into_inputs(self) -> Vec<McpStructuralStateInput> {
        match self {
            | Self::One(input) => vec![input],
            | Self::Many(inputs) => inputs,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
enum McpStructuralStateInput {
    Object(McpStructuralState),
    Compact(String),
}

impl McpStructuralStateInput {
    fn into_state(self) -> Result<StructuralStateFilter, String> {
        match self {
            | Self::Object(state) => {
                Ok(StructuralStateFilter { field: state.field, state: state.state.into() })
            }
            | Self::Compact(raw) => {
                StructuralStateFilter::from_str(&raw).map_err(|error| error.to_string())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct McpStructuralState {
    field: String,
    state: McpStructuralFieldState,
}
// sirno:witness:mcp-interface:end

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

// sirno:witness:mcp-interface:begin
#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct EntryWitnessParams {
    id: String,
    /// Return path and region as separate fields.
    #[serde(default, alias = "verbose-json")]
    verbose_json: bool,
}
// sirno:witness:mcp-interface:end

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

// sirno:witness:mist-commands:begin
#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct MistNameParams {
    /// Mist name. Omit for the default mist.
    mist: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct MistRenderParams {
    /// Mist name. Omit for the default mist.
    mist: Option<String>,
    #[serde(default)]
    dry: bool,
}
// sirno:witness:mist-commands:end

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct StatusParams {
    /// Select summary, normal, or full status detail.
    #[serde(default)]
    show: McpStatusMode,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum McpStatusMode {
    /// Include paths, counts, blockers, and message.
    #[default]
    Summary,
    /// Include compact status plus domain summaries.
    Normal,
    /// Include every status detail.
    Full,
}

impl From<McpStatusMode> for StatusMode {
    fn from(value: McpStatusMode) -> Self {
        match value {
            | McpStatusMode::Summary => Self::Summary,
            | McpStatusMode::Normal => Self::Normal,
            | McpStatusMode::Full => Self::Full,
        }
    }
}

// sirno:witness:upstream-commands:begin
#[derive(Clone, Debug, Deserialize, JsonSchema)]
struct UpstreamAddParams {
    /// Glacier domain used as the crystallized entry-address prefix.
    domain: String,
    /// Git URI or local Git repository source accepted by Git.
    git: String,
    /// Branch name to resolve.
    branch: Option<String>,
    /// Tag name to resolve.
    tag: Option<String>,
    /// Commit-ish to resolve.
    rev: Option<String>,
    /// Project root directory inside the Git tree.
    project: Option<PathBuf>,
    /// Project config manifest path relative to `project`.
    manifest: Option<PathBuf>,
    /// Upstream mist that selects the crystallized entries.
    mist: Option<String>,
}

impl UpstreamAddParams {
    fn into_request(self) -> Result<UpstreamAddRequest, String> {
        let ref_count = [self.branch.as_ref(), self.tag.as_ref(), self.rev.as_ref()]
            .into_iter()
            .flatten()
            .count();
        if ref_count != 1 {
            return Err("upstream add requires exactly one of branch, tag, or rev".to_owned());
        }
        let mut settings = if let Some(branch) = self.branch {
            UpstreamSettings::branch(self.git, branch)
        } else if let Some(tag) = self.tag {
            UpstreamSettings::tag(self.git, tag)
        } else if let Some(rev) = self.rev {
            UpstreamSettings::rev(self.git, rev)
        } else {
            unreachable!("checked upstream selector count")
        };
        if let Some(project) = self.project {
            settings.project = project;
        }
        if let Some(manifest) = self.manifest {
            settings.manifest = manifest;
        }
        if let Some(mist) = self.mist {
            settings.mist = Some(entry_atom(mist)?);
        }
        Ok(UpstreamAddRequest { domain: entry_atom(self.domain)?, settings })
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct UpstreamDomainsParams {
    /// Selected glacier domains. Empty means every upstream.
    #[serde(default)]
    domains: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct UpstreamCrystallizeParams {
    /// Selected glacier domains. Empty means every upstream.
    #[serde(default)]
    domains: Vec<String>,
    /// Use only existing lock records and cache mirrors.
    #[serde(default)]
    locked: bool,
}

impl UpstreamCrystallizeParams {
    fn into_request(self) -> Result<UpstreamCrystallizeRequest, String> {
        Ok(UpstreamCrystallizeRequest { domains: entry_atoms(self.domains)?, locked: self.locked })
    }
}
// sirno:witness:upstream-commands:end

#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
struct TideStatusParams {
    /// Select review entries, full open workitems, or all workitems.
    #[serde(default)]
    show: McpTideStatusMode,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum McpTideStatusMode {
    /// Return only entry addresses that need review.
    #[default]
    Review,
    /// Include full open workitem statuses.
    Full,
    /// Include full open and resolved workitem statuses.
    All,
}

impl From<McpTideStatusMode> for TideStatusMode {
    fn from(value: McpTideStatusMode) -> Self {
        match value {
            | McpTideStatusMode::Review => Self::Review,
            | McpTideStatusMode::Full => Self::Full,
            | McpTideStatusMode::All => Self::All,
        }
    }
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
            neighbors: self
                .neighbors
                .into_iter()
                .map(entry_address)
                .collect::<Result<Vec<_>, _>>()?,
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
            neighbors: self
                .neighbors
                .into_iter()
                .map(entry_address)
                .collect::<Result<Vec<_>, _>>()?,
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
            entry_address(value.ripple)?,
            value.field,
            StructuralEdgeDirection::from_str(&value.direction)
                .map_err(|error| error.to_string())?,
            entry_address(value.neighbor)?,
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
    use crate::{
        CONFIG_FILE_NAME, EntryAddress, MetaRegistry, RepoMember, RepoSettings, SirnoConfig,
        StructuralSettings,
    };

    // sirno:witness:mcp-interface:begin
    const EXPECTED_TOOLS: &[&str] = &[
        "sirno_anchor_check",
        "sirno_anchor_status",
        "sirno_anchor_update",
        "sirno_cwd",
        "sirno_entry_artifact_add",
        "sirno_entry_artifact_list",
        "sirno_entry_artifact_remove",
        "sirno_entry_artifact_rename",
        "sirno_entry_freeze",
        "sirno_entry_melt",
        "sirno_entry_new",
        "sirno_entry_path",
        "sirno_entry_query",
        "sirno_entry_read",
        "sirno_entry_rename",
        "sirno_entry_rg",
        "sirno_entry_witness",
        "sirno_lake_check",
        "sirno_lake_init",
        "sirno_lake_move",
        "sirno_mist_intake",
        "sirno_mist_render",
        "sirno_mist_render_delete",
        "sirno_mist_status",
        "sirno_status",
        "sirno_tide_reset",
        "sirno_tide_resolve",
        "sirno_tide_status",
        "sirno_tide_unresolve",
        "sirno_upstream_add",
        "sirno_upstream_crystallize",
        "sirno_upstream_remove",
        "sirno_upstream_status",
        "sirno_upstream_update",
    ];
    // sirno:witness:mcp-interface:end

    fn write_project(root: &Path) -> PathBuf {
        let config_path = root.join(CONFIG_FILE_NAME);
        let docs = root.join("docs");
        SirnoConfig::new("docs").write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        seed_meta_registry().write(root.join(".sirno").join("meta.toml")).unwrap();
        write_intrinsic_entries(&docs);
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

    fn write_witness_project(root: &Path) -> PathBuf {
        let config_path = root.join(CONFIG_FILE_NAME);
        let docs = root.join("docs");
        let src = root.join("src");
        SirnoConfig {
            repo: Some(RepoSettings { members: vec![RepoMember::new("src").unwrap()] }),
            ..SirnoConfig::new("docs")
        }
        .write_new(&config_path)
        .unwrap();
        fs::create_dir(&docs).unwrap();
        fs::create_dir(&src).unwrap();
        write_intrinsic_entries(&docs);
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
        let witness_source = format!(
            "{}{}{}\n{}\n{}{}{}\n",
            "// sirno",
            ":witness:",
            "alpha:begin",
            "pub fn alpha() {}",
            "// sirno",
            ":witness:",
            "alpha:end"
        );
        fs::write(src.join("lib.rs"), witness_source).unwrap();
        config_path
    }

    fn write_open_tide_project(root: &Path) -> PathBuf {
        let config_path = root.join(CONFIG_FILE_NAME);
        let docs = root.join("docs");
        let config = SirnoConfig::new("docs");
        config.write_new(&config_path).unwrap();
        fs::create_dir(&docs).unwrap();
        write_intrinsic_entries(&docs);
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
belongs:
  - beta
---

Body.
",
        )
        .unwrap();
        fs::write(
            docs.join("beta.md"),
            "\
---
name: Beta
desc: Beta entry.
---

Body.
",
        )
        .unwrap();
        fs::write(
            docs.join("belongs.md"),
            "\
---
name: Belongs
desc: A structural link relation.
meta.type: \"structural\"
meta.ripple.lake: [\"to\"]
---

Body.
",
        )
        .unwrap();
        fs::write(
            docs.join("alpha.md"),
            "\
---
name: Alpha
desc: Alpha entry.
belongs:
  - beta
---

Changed body.
",
        )
        .unwrap();
        config_path
    }

    fn write_intrinsic_entries(docs: &Path) {
        fs::write(
            docs.join("name.md"),
            "\
---
name: Name
desc: The required plain-string title field for entries.
meta.type: \"intrinsic\"
---

Body.
",
        )
        .unwrap();
        fs::write(
            docs.join("desc.md"),
            "\
---
name: Description
desc: The required plain-string summary field for entries.
meta.type: \"intrinsic\"
---

Body.
",
        )
        .unwrap();
    }

    fn seed_meta_registry() -> MetaRegistry {
        MetaRegistry::from_parts(
            [
                ("name", EntryAddress::new("name").unwrap()),
                ("desc", EntryAddress::new("desc").unwrap()),
            ],
            StructuralSettings::default(),
        )
    }

    fn structured(result: &CallToolResult) -> &serde_json::Value {
        result.structured_content.as_ref().expect("tool result has structured content")
    }

    fn intrinsic_params(
        fields: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> BTreeMap<String, String> {
        fields.into_iter().map(|(field, value)| (field.to_owned(), value.to_owned())).collect()
    }

    fn relation_params(
        fields: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> McpStructuralTargets {
        McpStructuralTargets(
            fields
                .into_iter()
                .map(|(field, target)| McpStructuralTarget {
                    field: field.to_owned(),
                    target: target.to_owned(),
                })
                .collect(),
        )
    }

    fn write_topic_relation(root: &Path) {
        fs::write(
            root.join("topic.md"),
            "\
---
name: Topic
desc: Test relation.
meta.type: \"structural\"
---

Body.
",
        )
        .unwrap();
    }

    #[test]
    fn entry_new_params_require_intrinsic_metadata() {
        let params = serde_json::from_value::<EntryNewParams>(json!({
            "id": "alpha",
            "structural": [],
            "body": "Body."
        }));

        assert!(params.is_err());
    }

    #[test]
    fn tool_router_exposes_grouped_tool_surface() {
        let server = SirnoMcpServer::new(SurfaceContext::new("Sirno.toml"));
        let names = server
            .tool_router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();

        assert_eq!(names, EXPECTED_TOOLS);
        // sirno:witness:mcp-interface:begin
        assert!(names.iter().all(|name| !name.starts_with("sirno_util_")));
        // sirno:witness:mcp-interface:end
    }

    #[test]
    fn direct_tool_call_returns_structured_content_and_summary_text() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let server = SirnoMcpServer::new(SurfaceContext::new(&config_path));

        let cwd = server.cwd(Parameters(CwdParams::default())).unwrap();
        let init = server
            .lake_init(Parameters(LakeInitParams { lake: Some(PathBuf::from("docs")) }))
            .unwrap();
        write_topic_relation(&temp.path().join("docs"));
        let entry = server
            .entry_new(Parameters(EntryNewParams {
                id: "alpha".to_owned(),
                intrinsic: intrinsic_params([("name", "Alpha"), ("desc", "Alpha entry.")]),
                structural: relation_params([("topic", "concept")]),
                body: Some("Body.".to_owned()),
            }))
            .unwrap();
        let read = server
            .entry_read(Parameters(EntryReadParams {
                id: "alpha".to_owned(),
                content: McpEntryReadContent::default(),
            }))
            .unwrap();
        let text = entry
            .content
            .first()
            .and_then(|content| content.as_text())
            .map(|text| text.text.as_str())
            .unwrap();

        assert_eq!(structured(&cwd)["ok"], true);
        assert_eq!(structured(&cwd)["changed"], false);
        assert!(structured(&cwd)["path"].as_str().is_some_and(|path| !path.is_empty()));
        assert_eq!(structured(&init)["ok"], true);
        assert_eq!(structured(&entry)["id"], "alpha");
        assert_eq!(structured(&read)["intrinsic"]["name"], "Alpha");
        assert_eq!(structured(&read)["intrinsic"]["desc"], "Alpha entry.");
        assert_eq!(structured(&read)["relation"]["topic"][0], "concept");
        assert!(structured(&read).get("name").is_none());
        assert!(structured(&read).get("desc").is_none());
        assert_eq!(structured(&read)["body"], "Body.");
        assert!(structured(&read).get("source").is_none());
        assert!(text.starts_with("created "));
        assert!(text.contains("alpha.md"));
        assert!(!text.contains("\"ok\""));
    }

    #[test]
    fn entry_read_content_selector_controls_large_fields() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        let server = SirnoMcpServer::new(SurfaceContext::new(&config_path));

        server.lake_init(Parameters(LakeInitParams { lake: Some(PathBuf::from("docs")) })).unwrap();
        write_topic_relation(&temp.path().join("docs"));
        server
            .entry_new(Parameters(EntryNewParams {
                id: "alpha".to_owned(),
                intrinsic: intrinsic_params([("name", "Alpha"), ("desc", "Alpha entry.")]),
                structural: relation_params([("topic", "concept")]),
                body: Some("Body.".to_owned()),
            }))
            .unwrap();

        let omitted: EntryReadParams = serde_json::from_value(json!({ "id": "alpha" })).unwrap();
        let metadata = server
            .entry_read(Parameters(EntryReadParams {
                id: "alpha".to_owned(),
                content: McpEntryReadContent::Metadata,
            }))
            .unwrap();
        let body = server.entry_read(Parameters(omitted)).unwrap();
        let source = server
            .entry_read(Parameters(EntryReadParams {
                id: "alpha".to_owned(),
                content: McpEntryReadContent::Source,
            }))
            .unwrap();
        let full = server
            .entry_read(Parameters(EntryReadParams {
                id: "alpha".to_owned(),
                content: McpEntryReadContent::Full,
            }))
            .unwrap();

        assert_eq!(structured(&metadata)["id"], "alpha");
        assert_eq!(structured(&metadata)["intrinsic"]["name"], "Alpha");
        assert_eq!(structured(&metadata)["intrinsic"]["desc"], "Alpha entry.");
        assert_eq!(structured(&metadata)["relation"]["topic"][0], "concept");
        assert!(structured(&metadata).get("body").is_none());
        assert!(structured(&metadata).get("source").is_none());
        assert!(structured(&metadata).get("name").is_none());
        assert!(structured(&metadata).get("desc").is_none());
        assert_eq!(structured(&body)["body"], "Body.");
        assert!(structured(&body).get("source").is_none());
        assert!(structured(&source).get("body").is_none());
        assert!(structured(&source)["source"].as_str().unwrap().contains("desc: Alpha entry."));
        assert_eq!(structured(&full)["body"], "Body.");
        assert!(structured(&full)["source"].as_str().unwrap().contains("desc: Alpha entry."));
    }

    #[test]
    fn query_params_accept_compact_structural_filters() {
        let params: EntryQueryParams = serde_json::from_value(json!({
            "has": "belongs=agent-skills",
            "is": "category=present",
        }))
        .unwrap();
        let filters = params.has.into_filters().unwrap();
        let states = params.is.into_states().unwrap();

        assert_eq!(filters[0].field, "belongs");
        assert_eq!(filters[0].targets[0].as_str(), "agent-skills");
        assert_eq!(states[0].field, "category");
        assert!(matches!(states[0].state, StructuralFieldState::Present));
    }

    #[test]
    fn query_columns_distinguish_omitted_empty_and_selected() {
        assert!(matches!(query_columns(None).unwrap(), QueryColumnSelection::Default));
        assert!(matches!(query_columns(Some(Vec::new())).unwrap(), QueryColumnSelection::Options));

        let QueryColumnSelection::Selected(columns) =
            query_columns(Some(vec!["id".to_owned(), "category".to_owned()])).unwrap()
        else {
            panic!("expected selected query columns");
        };

        assert_eq!(columns.labels(), vec!["id", "category"]);
    }

    #[test]
    fn status_defaults_to_summary_detail() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = write_project(temp.path());
        let server = SirnoMcpServer::new(SurfaceContext::new(config_path));

        let summary = server.status(Parameters(StatusParams::default())).unwrap();
        let normal =
            server.status(Parameters(StatusParams { show: McpStatusMode::Normal })).unwrap();
        let full = server.status(Parameters(StatusParams { show: McpStatusMode::Full })).unwrap();

        assert_eq!(structured(&summary)["ok"], true);
        assert_eq!(structured(&summary)["entry_count"], 3);
        assert_eq!(structured(&summary)["blockers"]["check_errors"], 0);
        assert!(structured(&summary)["message"].as_str().unwrap().starts_with("status clean:"));
        assert!(structured(&summary).get("check_policy").is_none());
        assert!(structured(&summary).get("structural_fields").is_none());
        assert!(structured(&summary).get("tide").is_none());
        assert!(structured(&summary).get("mist").is_none());
        assert!(structured(&summary).get("check").is_none());

        assert_eq!(structured(&normal)["check_policy"]["mode"], "review");
        assert!(structured(&normal).get("tide").is_some());
        assert!(structured(&normal).get("mist").is_some());
        assert!(structured(&normal).get("structural_fields").is_none());
        assert!(structured(&normal).get("check").is_none());

        assert_eq!(structured(&full)["check"]["ok"], true);
        let structural_field_count =
            structured(&full)["structural_field_count"].as_u64().unwrap() as usize;
        if structural_field_count == 0 {
            assert!(structured(&full).get("structural_fields").is_none());
        } else {
            assert_eq!(
                structured(&full)["structural_fields"].as_array().unwrap().len(),
                structural_field_count
            );
        }
    }

    #[test]
    fn tide_status_defaults_to_review_entries() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = write_open_tide_project(temp.path());
        let server = SirnoMcpServer::new(SurfaceContext::new(config_path));

        let summary = server.tide_status(Parameters(TideStatusParams::default())).unwrap();
        let full = server
            .tide_status(Parameters(TideStatusParams { show: McpTideStatusMode::Full }))
            .unwrap();
        let all = server
            .tide_status(Parameters(TideStatusParams { show: McpTideStatusMode::All }))
            .unwrap();

        assert_eq!(structured(&summary)["ok"], false);
        assert_eq!(structured(&summary)["review_entries"], json!(["beta"]));
        assert!(structured(&summary).get("statuses").is_none());
        assert_eq!(structured(&full)["review_entries"], json!(["beta"]));
        assert_eq!(structured(&full)["statuses"][0]["workitem"]["neighbor"], "beta");
        assert_eq!(structured(&all)["statuses"][0]["workitem"]["neighbor"], "beta");
    }

    #[test]
    fn entry_witness_defaults_to_location_and_body() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = write_witness_project(temp.path());
        let server = SirnoMcpServer::new(SurfaceContext::new(config_path));

        let result = server
            .entry_witness(Parameters(EntryWitnessParams {
                id: "alpha".to_owned(),
                verbose_json: false,
            }))
            .unwrap();
        let record = &structured(&result)["records"][0];

        assert!(record["location"].as_str().unwrap().contains("src/lib.rs:1:1-3:"));
        assert!(record["body"].as_str().unwrap().contains("pub fn alpha() {}"));
        assert!(record.get("path").is_none());
        assert!(record.get("region").is_none());
        assert!(record.get("opening").is_none());
        assert!(record.get("closing").is_none());
        assert!(record.get("marker").is_none());
    }

    #[test]
    fn entry_witness_verbose_json_keeps_path_and_region() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = write_witness_project(temp.path());
        let server = SirnoMcpServer::new(SurfaceContext::new(config_path));

        let result = server
            .entry_witness(Parameters(EntryWitnessParams {
                id: "alpha".to_owned(),
                verbose_json: true,
            }))
            .unwrap();
        let record = &structured(&result)["records"][0];

        assert!(record["path"].as_str().unwrap().ends_with("src/lib.rs"));
        assert_eq!(record["region"]["start_line"], json!(1));
        assert!(record["body"].as_str().unwrap().contains("pub fn alpha() {}"));
        assert!(record.get("location").is_none());
        assert!(record.get("opening").is_none());
        assert!(record.get("closing").is_none());
        assert!(record.get("marker").is_none());
    }

    #[test]
    fn entry_witness_accepts_verbose_json_flag_spelling() {
        let params: EntryWitnessParams = serde_json::from_value(json!({
            "id": "alpha",
            "verbose-json": true,
        }))
        .unwrap();

        assert!(params.verbose_json);
    }

    #[derive(Clone, Debug, Default)]
    struct DummyClient;

    impl ClientHandler for DummyClient {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    #[tokio::test]
    async fn stdio_smoke_lists_tools_and_calls_status() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = write_project(temp.path());
        let server = SirnoMcpServer::new(SurfaceContext::new(config_path));
        let (server_transport, client_transport) = tokio::io::duplex(8192);

        let server_handle = tokio::spawn(async move {
            server.serve(server_transport).await.unwrap().waiting().await.unwrap();
        });
        let client = DummyClient.serve(client_transport).await.unwrap();

        let tools = client.peer().list_tools(None).await.unwrap();
        assert_eq!(tools.tools.len(), EXPECTED_TOOLS.len());
        assert!(tools.tools.iter().any(|tool| tool.name == "sirno_status"));

        let resources = client.peer().list_resources(None).await.unwrap();
        assert_eq!(resources.resources.len(), SKILL_RESOURCES.len());
        assert!(resources.resources.iter().any(|resource| {
            resource.uri == "sirno://skills/sirno-editor"
                && resource.mime_type.as_deref() == Some(SKILL_RESOURCE_MIME_TYPE)
        }));
        assert!(resources.resources.iter().any(|resource| {
            resource.uri == "sirno://skills/sirno-actualizer"
                && resource.mime_type.as_deref() == Some(SKILL_RESOURCE_MIME_TYPE)
        }));
        assert!(resources.resources.iter().any(|resource| {
            resource.uri == "sirno://skills/sirno-internalizer"
                && resource.mime_type.as_deref() == Some(SKILL_RESOURCE_MIME_TYPE)
        }));
        assert!(resources.resources.iter().any(|resource| {
            resource.uri == "sirno://skills/sirno-finalizer"
                && resource.mime_type.as_deref() == Some(SKILL_RESOURCE_MIME_TYPE)
        }));
        assert!(resources.resources.iter().any(|resource| {
            resource.uri == "sirno://skills/design-doc-writer"
                && resource.mime_type.as_deref() == Some(SKILL_RESOURCE_MIME_TYPE)
        }));

        let resource_templates = client.peer().list_resource_templates(None).await.unwrap();
        assert_eq!(resource_templates.resource_templates.len(), 1);
        assert_eq!(
            resource_templates.resource_templates[0].raw.uri_template,
            ENTRY_RESOURCE_URI_TEMPLATE
        );

        let skill = client
            .peer()
            .read_resource(ReadResourceRequestParams::new("sirno://skills/sirno-editor"))
            .await
            .unwrap();
        let Some(ResourceContents::TextResourceContents { text, mime_type, .. }) =
            skill.contents.first()
        else {
            panic!("expected text skill resource");
        };
        assert_eq!(mime_type.as_deref(), Some(SKILL_RESOURCE_MIME_TYPE));
        assert!(text.contains("# Sirno Editor"));
        assert!(text.contains("## Workflow"));
        assert!(text.contains("Meta registry:"));
        assert!(text.contains(".sirno/meta.toml"));
        assert!(text.contains("- `desc` defined by `desc`"));
        assert!(!text.contains(SKILL_PROJECT_CONTEXT_TOKEN));

        let design_skill = client
            .peer()
            .read_resource(ReadResourceRequestParams::new("sirno://skills/design-doc-writer"))
            .await
            .unwrap();
        let Some(ResourceContents::TextResourceContents { text, .. }) =
            design_skill.contents.first()
        else {
            panic!("expected text design-doc-writer resource");
        };
        assert!(text.contains("# Design Doc Writer"));
        assert!(text.contains("## Reader Evaluation"));

        let entry = client
            .peer()
            .read_resource(ReadResourceRequestParams::new("sirno://entries/alpha"))
            .await
            .unwrap();
        let Some(ResourceContents::TextResourceContents { text, mime_type, .. }) =
            entry.contents.first()
        else {
            panic!("expected text entry resource");
        };
        assert_eq!(mime_type.as_deref(), Some(ENTRY_RESOURCE_MIME_TYPE));
        assert!(text.contains("name: Alpha"));
        assert!(text.contains("Body."));

        let result = client
            .peer()
            .call_tool(
                CallToolRequestParams::new("sirno_status")
                    .with_arguments(json!({}).as_object().unwrap().clone()),
            )
            .await
            .unwrap();

        let status = result.structured_content.as_ref().unwrap();
        assert_eq!(status["ok"], true);
        assert_eq!(status["entry_count"], 3);
        assert_eq!(status["blockers"]["check_errors"], 0);
        assert!(status["message"].as_str().unwrap().starts_with("status clean:"));
        assert!(status.get("check_policy").is_none());
        assert!(status.get("anchor").is_none());

        let full_result = client
            .peer()
            .call_tool(
                CallToolRequestParams::new("sirno_status")
                    .with_arguments(json!({ "show": "full" }).as_object().unwrap().clone()),
            )
            .await
            .unwrap();
        let full_status = full_result.structured_content.as_ref().unwrap();
        assert_eq!(full_status["check_policy"]["mode"], "review");
        assert_eq!(full_status["check"]["ok"], true);

        let cwd = client
            .peer()
            .call_tool(
                CallToolRequestParams::new("sirno_cwd")
                    .with_arguments(json!({}).as_object().unwrap().clone()),
            )
            .await
            .unwrap();
        assert_eq!(cwd.structured_content.as_ref().unwrap()["ok"], true);
        assert_eq!(cwd.structured_content.as_ref().unwrap()["changed"], false);

        client.cancel().await.unwrap();
        server_handle.await.unwrap();
    }
}
