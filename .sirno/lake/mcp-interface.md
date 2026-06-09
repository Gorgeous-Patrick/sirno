---
name: MCP Interface
desc: Agent-facing resources, tools, JSON behavior, and surface ownership.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

The MCP interface exposes grouped project command tools and skill resources.
It does not expose top-level CLI aliases, shortcut commands, prompts, or CLI utility commands.
MCP tools should call typed surface helpers
and prefer JSON rendering through the shared serializer.

MCP is the agent-facing project interface:

- it serves stable project operations
- it serves lake-owned skill instructions as resources
- it keeps host setup and package maintenance as explicit human CLI actions

Skill resources are `text/markdown` payloads rendered by `src/mcp.rs`
from lake-owned `SKILL.full.template.md` artifacts.
Templates may carry a runtime metadata slot.
When present,
MCP fills it with the configured lake path,
the default query columns,
and the active project's intrinsic and structural fields from `.sirno/meta.toml`.
Packaged `.agents/skills/sirno-*` wrappers tell agents to read these resources.

MCP resources are:

- `sirno://skills/design-doc-writer`
- `sirno://skills/sirno-editor`
- `sirno://skills/sirno-actualizer`
- `sirno://skills/sirno-internalizer`
- `sirno://skills/sirno-narrative-session`
- `sirno://skills/sirno-skill-synthesizer`
- `sirno://skills/sirno-curator`
- `sirno://skills/sirno-finalizer`
- `sirno://entries/{id}` through the entry resource template

Reading one entry resource returns the full stored Markdown source as `text/markdown`.

MCP tool names are stable snake-case names prefixed with `sirno_`:

- project binding: `sirno_cwd`, `sirno_status`
- entries: `sirno_entry_new`, `sirno_entry_rename`, `sirno_entry_freeze`,
  `sirno_entry_melt`, `sirno_entry_path`, `sirno_entry_read`, `sirno_entry_query`,
  `sirno_entry_rg`, and `sirno_entry_witness`
- entry artifacts: `sirno_entry_artifact_list`, `sirno_entry_artifact_add`,
  `sirno_entry_artifact_rename`, and `sirno_entry_artifact_remove`
- lake: `sirno_lake_init`, `sirno_lake_move`, and `sirno_lake_check`
- mist: `sirno_mist_status`, `sirno_mist_intake`, `sirno_mist_render`,
  and `sirno_mist_render_delete`
- anchor: `sirno_anchor_status`, `sirno_anchor_check`, and `sirno_anchor_update`
- tide: `sirno_tide_status`, `sirno_tide_resolve`, `sirno_tide_unresolve`,
  and `sirno_tide_reset`

MCP tools accept typed JSON arguments.
`sirno_cwd` accepts optional `{ path }`.
With `path`, it changes the process current working directory before returning it.
Without `path`, it returns the current working directory without changing it.
Relative config paths are resolved against the process current working directory
on every project tool call.

`sirno_entry_read` returns parsed metadata and selected entry content.
Its `content` selector accepts `metadata`, `body`, `source`, or `full`.
Omitting `content` selects `body`,
so the default result includes parsed metadata and body text.
The full stored Markdown source is returned only for `source` or `full`.
Structural filters may use `{ field, targets }` objects
or compact `FIELD=ENTRY_ADDRESS[,ENTRY_ADDRESS]` strings.
Structural states may use `{ field, state }` objects
or compact `FIELD=present`, `FIELD=empty`, and `FIELD=missing` strings.
`sirno_entry_query` omits `columns` to select the default `id` and `path` columns.
An empty `columns` array returns selectable column names and no records.
A non-empty `columns` array selects `id`, `path`, discovered intrinsic fields,
or structural link relations.
`sirno_status` accepts `show: summary | normal | full`.
Omitting `show` selects `summary`,
which returns project paths, entry and structural-field counts,
blocker counts, and a short message.
`normal` adds check policy, Tide summary, and default mist status.
`full` adds structural link policy and review-mode check output.
`sirno_entry_rg` accepts `args: string[]` and returns captured `exit_code`, `stdout`, and `stderr`.
`sirno_entry_witness` accepts `{ id }` by default.
Default records contain `entry`, `location`, and `body`.
The `verbose_json` (`--verbose-json`) option keeps separate `path` and `region` fields.
Delimiter spans stay internal and CLI-oriented.

Successful tool calls return structured JSON content as their authoritative result.
They may also include concise text content for clients that read only text.
Text content summarizes the result instead of mirroring the full JSON payload.
Domain failures such as failed checks, dirty query preconditions,
and nonzero `rg` exits return structured results with `ok: false`.
Command failures return MCP tool errors with concise text.

The MCP surface calls `sirno::surface` methods for command behavior.
Public request and result DTOs live in `sirno::surface`.
The MCP interface only converts JSON parameters into surface requests
and surface DTOs into MCP tool results.
This keeps CLI JSON and MCP JSON aligned without duplicating command logic.

The MCP interface serves interactive tooling.
It can expose the same lake model to agents and editors
without asking them to shell out for every action.
