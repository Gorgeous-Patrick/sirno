---
name: Utility Commands
desc: Local operator commands for config repair, entry defaults, skills, and MCP startup.
category:
  - concept
belongs:
  - project-config
prerequisite:
  - project-config
refines:
  - command-families
---

The `sirno util` command family is the local operator and integration-maintenance surface.
It prepares or repairs the environment around a Sirno project.
Humans perform that operator work through the CLI.
MCP does not expose utility commands.

Utility terminal UIs share a main selectable table and a bottom key/message footer.
`j`, `k`, Up, and Down move the selected row.
`q` and Esc exit.

`sirno util completion` emits shell completion scripts.
Completion generation is a utility command,
not a lake operation.

`sirno util config` and `sirno util config tui` open an interactive terminal UI
for `Sirno.toml`.
Each row is a top-level config section with its presence status
and canonical-comment status.
`i` inserts the selected section with canonical comments.
`f` repairs comments for the selected non-empty section.

`sirno util config check` runs the non-interactive comment check.
It prints missing comments before the summary line,
does not write `Sirno.toml`,
and exits with failure when comments are missing.
`sirno util config fix` runs the non-interactive comment repair.
It rewrites `Sirno.toml` through the canonical config renderer when comments are missing.
Config utility commands reject `--lake-path`.

`sirno util entry` and `sirno util entry tui` open an interactive terminal UI
for common entry defaults.
Each row is a default entry address with its presence status
and the structural link relations that would be written under the current lake.
`i` inserts the selected missing entry.
`a` inserts all missing defaults.

The entry defaults include category vocabulary such as `category`, `meta`, `concept`,
and `narrative`.
They also include structural vocabulary such as `structural`, `belongs`, `refines`,
and `prerequisite`.
The entry utility accepts `--lake-path`.

`sirno util skills init` installs bundled Sirno skill wrappers
to their `.agents/skills/sirno-*` package targets.
`sirno util skills check` reports whether installed wrappers match those bundled constants.
`sirno util skills list` lists the bundled skill names and package targets.
`sirno util skills` and `sirno util skills tui` open an interactive terminal UI
for skill wrapper maintenance.
It checks installed wrappers on entry and shows wrapper and link records in a table.
`c` refreshes the check.
`i` installs or repairs the displayed wrappers and links.
`l` toggles `.claude/skills/sirno-*` link rows.
The `--claude-skills` option includes `.claude/skills/sirno-*` links in `init`, `check`,
`list`, and `tui` output.
Non-interactive skill utility commands print wrapper records as a table,
followed by a summary line.
Skill utility commands reject `--lake-path`.

`sirno util mcp --config PATH` starts the MCP server over stdio.
When `--config` is omitted, the server uses the default `Sirno.toml` path.
Project tools resolve that config path on each tool call.
If the config path is relative, the server process current working directory controls the project.
`sirno util mcp` rejects `--lake-path`;
the configured project selects its lake.
