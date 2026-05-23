---
name: MCP Interface
desc: Agent-facing resources, tools, JSON behavior, and adapter ownership.
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

Skill resources are `text/markdown` payloads embedded by `src/mcp.rs`
from the lake-owned `SKILL.full.md` artifacts.
Packaged `.agents/skills/sirno-*` wrappers tell agents to read these resources.

MCP resources are:

- `sirno://skills/design-doc-writer`
- `sirno://skills/sirno-editor`
- `sirno://skills/sirno-actualizer`
- `sirno://skills/sirno-internalizer`
- `sirno://skills/sirno-narrative-session`
- `sirno://skills/sirno-skill-synthesizer`
- `sirno://skills/sirno-curator`
- `sirno://entries/{id}` through the entry resource template

Reading one entry resource returns the full stored Markdown source as `text/markdown`.

MCP tool names are stable snake-case names prefixed with `sirno_`:

- project binding: `sirno_cwd`, `sirno_status`
- entries: `sirno_entry_new`, `sirno_entry_rename`, `sirno_entry_freeze`,
  `sirno_entry_melt`, `sirno_entry_path`, `sirno_entry_read`, `sirno_entry_query`,
  `sirno_entry_rg`, and `sirno_entry_witness`
- entry artifacts: `sirno_entry_artifact_list`, `sirno_entry_artifact_add`,
  `sirno_entry_artifact_rename`, and `sirno_entry_artifact_remove`
- lake: `sirno_lake_init`, `sirno_lake_move`, `sirno_lake_check`,
  `sirno_lake_render`, and `sirno_lake_render_delete`
- frost: `sirno_frost_init`, `sirno_frost_move`, `sirno_frost_commit`,
  `sirno_frost_checkout`, and `sirno_frost_defrost`
- tide: `sirno_tide_status`, `sirno_tide_resolve`, `sirno_tide_unresolve`,
  and `sirno_tide_reset`

MCP tools accept typed JSON arguments.
`sirno_cwd` accepts optional `{ path }`.
With `path`, it changes the process current working directory before returning it.
Without `path`, it returns the current working directory without changing it.
Relative config paths are resolved against the process current working directory
on every project tool call.

`sirno_entry_read` returns parsed metadata, body text, and the full stored Markdown source.
Structural filters may use `{ field, targets }` objects
or compact `FIELD=ENTRY_ADDRESS[,ENTRY_ADDRESS]` strings.
Structural states may use `{ field, state }` objects
or compact `FIELD=present`, `FIELD=empty`, and `FIELD=missing` strings.
`sirno_entry_query` omits `columns` to select the default `id` and `name` columns.
An empty `columns` array returns selectable column names and no records.
A non-empty `columns` array selects built-in columns and configured link relations.
`sirno_status` returns typed frost, check-policy, structural-edge,
tide, and commit-readiness objects.
`sirno_entry_rg` accepts `args: string[]` and returns captured `exit_code`, `stdout`, and `stderr`.
`sirno_entry_witness` accepts `{ id }` by default.
Default records contain `entry`, `location`, and `body`.
The `verbose_json` (`--verbose-json`) option keeps separate `path` and `region` fields.
Delimiter spans stay internal and CLI-oriented.

Successful tool calls return structured JSON content.
They also include the same JSON as pretty text content for clients that read only text.
Domain failures such as failed checks, dirty query preconditions,
and nonzero `rg` exits return structured results with `ok: false`.
Command failures return MCP tool errors with concise text.

The MCP adapter calls `sirno::surface` methods for command behavior.
Public request and result DTOs live in `sirno::surface`.
The adapter only converts JSON parameters into surface requests
and surface DTOs into MCP tool results.
This keeps CLI JSON and MCP JSON aligned without duplicating command logic.

The MCP interface serves interactive tooling.
It can expose the same lake model to agents and editors
without asking them to shell out for every action.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
