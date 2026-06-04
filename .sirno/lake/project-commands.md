---
name: Project Commands
desc: Project status and setup commands for a Sirno-managed repository.
category:
  - concept
belongs:
  - project-config
prerequisite:
  - project-config
---

Project commands cover project-wide status and setup.
Domain operations live with the design object they operate on.

## Status

`sirno status` and `sirno st` summarize the configured project as an operational dashboard.
Status reports:

- config path;
- reservoir path;
- entry count;
- structural link relation count;
- review-mode lake check;
- active tide summary;
- default mist projection state.

CLI status keeps structural policy collapsed.
MCP status returns typed structural link policy, tide state, and mist projection state.

## Setup

| Command | Behavior |
|---|---|
| `sirno init` | Opens an interactive setup flow. |
| `sirno init --all` | Runs full setup without prompts. |
| `sirno lake init [PATH]` | Creates config and seed entries. |

Interactive init asks which setup parts to run,
asks for default paths when no path flag supplies them,
asks whether installed wrappers should be linked into Claude skills,
shows the init plan,
and applies it after confirmation.

Full setup creates a Sirno config,
ordinary seed entries in the reservoir,
and packaged skill wrappers.
The default reservoir path is `.sirno/lake` next to `Sirno.toml`.
The default misty workspace renders to `sirno-lake/`.

Setup flags:

| Flag | Behavior |
|---|---|
| `--lake PATH` | Chooses a non-default reservoir path. |
| `--no-lake` | Skips lake setup. |
| `--no-skills` | Skips packaged skill wrappers. |
| `--claude-skills` | Links `.claude/skills/sirno-*` to installed wrappers. |

The config is still written when another selected setup part needs it.
When a setup part is skipped,
its path option is not accepted.

## Domain Command Homes

| Entry | Command area |
|---|---|
| `lake-commands` | Lake initialization, checking, and storage movement. |
| `sirno-anchor` | Anchor status, check, and update operations. |
| `sirno-upstream` | Upstream declaration, crystallization, update, and status. |
| `structural-check` | Structural checking and check-mode selection. |
| `mist` | Mist status, intake, render, and generated navigation commands. |
