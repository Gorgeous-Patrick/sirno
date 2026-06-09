---
name: CLI Interface
desc: Human command grammar and output conventions for Sirno.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

The CLI is the human-facing operational interface.
Human interaction with Sirno should happen through CLI commands.
The binary `main.rs` delegates process startup to `sirno::surface`.
`sirno --version` prints the package version from `Cargo.toml` before command dispatch.

Reusable helpers in `sirno::surface` return typed query, path, tide, and witness data
before the CLI renders human text or JSON.
When a command displays entry metadata,
it renders discovered intrinsic fields instead of fixed title or summary fields.
Human output prints records, tables, or diagnostics before command summary lines.
Diagnostics print whenever a command reports a failure, independent of detail level.
Commands with no detail may print only their summary.
Terminal output may color semantic labels such as setup choices, diagnostic severity,
check state, tide state, and wrapper status.
JSON output remains structured data and carries no terminal styling.

Human-facing usage and mechanism documents should spell Sirno operations as CLI commands.
Agent-facing discipline entries, packaged skill resources, and MCP documentation
should spell Sirno operations as MCP tools when the agent performs them.

The global `-C, --config PATH` option selects the Sirno project config file.
The global `-L, --lake-path PATH` option overrides the configured lake
for commands that read or write the active lake.
The global `-q, --quiet` flag mutes diagnostic detail blocks in human output,
keeping summary lines.

Common command aliases keep terminal use compact:
`q` for `query`, `st` for `status`, and `w` or `wit` for `witness`.
`sirno status` is the go-to read-only health command.
Its `--mode edit | review` option selects the check boundary.

Entry commands live under `sirno entry`.
Selected common entry operations also have top-level wrappers.
Lake-wide operations also live under `sirno lake`.
Entry artifact operations also have the top-level `sirno artifact` form.
Anchor baseline updates live under `sirno anchor`.

When a selected top-level command delegates to a group,
the grouped spelling uses the same subcommands and aliases.
For example, `sirno query`, `sirno q`, `sirno entry query`, and `sirno entry q`
select the same entry operation.

Top-level `sirno move` groups mutation moves:
`sirno move entry OLD_ENTRY_ADDRESS NEW_ENTRY_ADDRESS`,
and `sirno move lake PATH`.
`sirno mv ...` is its short form.
Each wrapper delegates to the corresponding grouped move command.

For artifact mutation,
`sirno artifact ...` and `sirno entry artifact ...` select the same entry operation.
CLI commands should remain plain enough to use from a terminal
and stable enough to share behavior with MCP tools.
