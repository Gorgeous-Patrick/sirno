---
name: Utility Commands
desc: Local operator commands for config repair, entry defaults, skills, and MCP startup.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

The `sirno util` command family is the local operator and integration-maintenance surface.
It prepares or repairs the environment around a Sirno project.
Humans perform that operator work through the CLI.
MCP does not expose utility commands.

`sirno util completion` emits shell completion scripts.
Completion generation is a utility command,
not a lake operation.

`sirno util config` opens an interactive terminal UI for `Sirno.toml`.
Each row is a top-level config section with its presence status
and canonical-comment status.
`j`, `k`, Up, and Down move the selected row.
`i` inserts the selected section with canonical comments.
`f` repairs comments for the selected non-empty section.
`q` and Esc exit.

`sirno util config --dry` keeps the non-interactive comment check.
It prints missing comments before the summary line,
does not write `Sirno.toml`,
and exits with failure when comments are missing.
`sirno util config --fix` keeps the non-interactive comment repair.
It rewrites `Sirno.toml` through the canonical config renderer when comments are missing.
`--dry` and `--fix` are mutually exclusive.
Config utility commands reject `--lake-path` and `--frost-path`.

`sirno util entry` opens an interactive terminal UI for common entry defaults.
Each row is a default entry id with its presence status
and the structural fields that would be written under the current `Sirno.toml`.
`j`, `k`, Up, and Down move the selected row.
`i` inserts the selected missing entry.
`a` inserts all missing defaults.
`q` and Esc exit.

The entry defaults include category vocabulary such as `category`, `meta`, `concept`,
and `narrative`.
They also include structural vocabulary such as `structural`, `belongs`, `refines`,
and `prerequisite`.
The utility accepts `--lake-path` and rejects `--frost-path`.

`sirno util skills init` installs bundled Sirno skill wrappers
to their `.agents/skills/sirno-*` package targets.
`sirno util skills check` reports whether installed wrappers match those bundled constants.
`sirno util skills list` lists the bundled skill names and package targets.
The `--claude-skills` option includes `.claude/skills/sirno-*` links in `init`, `check`,
and `list` output.
Skill utility commands print wrapper records as a table,
followed by a summary line.
Skill utility commands reject `--lake-path` and `--frost-path`.

`sirno util mcp --config PATH` starts the MCP server over stdio.
When `--config` is omitted, the server uses the default `Sirno.toml` path.
Project tools resolve that config path on each tool call.
If the config path is relative, the server process current working directory controls the project.
`sirno util mcp` rejects `--lake-path` and `--frost-path`;
the configured project selects its lake and optional frost path.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
