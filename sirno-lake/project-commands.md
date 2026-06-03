---
name: Project Commands
desc: CLI commands for project setup, storage movement, Anchor, checks, and rendering.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

Project commands operate on the configured project, active lake, Anchor baseline,
and generated footer state.

## Status

`sirno status` and `sirno st` summarize the configured project as an operational dashboard.
Status reports:

- config path;
- lake path;
- entry count;
- Anchor ripple state;
- structural link relation count;
- review-mode lake check;
- active tide summary.

CLI status keeps structural policy collapsed.
MCP status returns the effective typed structural link policy,
including rendered directions from config and tide directions from relation entries.

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
ordinary seed entries,
and packaged skill wrappers.
Default paths are derived from the directory that contains `Sirno.toml`:

- `<repo>-lake` for the lake path.

Setup flags:

| Flag | Behavior |
|---|---|
| `--lake PATH` | Chooses a non-default lake path. |
| `--no-lake` | Skips lake setup. |
| `--no-skills` | Skips packaged skill wrappers. |
| `--claude-skills` | Links `.claude/skills/sirno-*` to installed wrappers. |

The config is still written when another selected setup part needs it.
When a setup part is skipped,
its path option is not accepted.

## Storage Movement

| Command | Behavior |
|---|---|
| `sirno lake move PATH` | Changes `[lake].path` and renames the lake directory. |
| `sirno lake mv PATH` | Short form of `sirno lake move PATH`. |
| `sirno move lake PATH` | Top-level wrapper for the same lake move. |
| `sirno mv lake PATH` | Short top-level wrapper for the same lake move. |

Path moves create missing destination parents and refuse existing destinations.
A destination inside the moved path is handled through temporary staging.

## Anchor

| Command | Behavior |
|---|---|
| `sirno anchor status` | Shows current lake ripples against `.sirno/anchor.toml`. |
| `sirno anchor check` | Validates `.sirno/anchor.toml` and compares it with the lake. |
| `sirno anchor update` | Accepts the current lake as the new Anchor baseline. |

Anchor update fails while open tide workitems remain.
When `[tutorial]` is present,
this failure can include tutorial text controlled by `[tutorial].anchor_update_tide`
and `[tutorial].anchor_bootstrap_tide`.

The first implementation stores review resolutions in `Sirno.lock.toml`.
The target design moves active review status to `.sirno/tide.toml`.

## Upstreams

| Command | Behavior |
|---|---|
| `sirno upstream add DOMAIN --git SOURCE ...` | Declares and crystallizes a Git upstream lake. |
| `sirno upstream remove DOMAIN` | Removes the declaration and managed glacier content. |
| `sirno upstream crystallize [DOMAIN]` | Crystallizes upstreams into glaciers. |
| `sirno upstream crystallize [DOMAIN] --locked` | Uses only existing locks and cache mirrors. |
| `sirno upstream update [DOMAIN]` | Refreshes upstream locks and glacier content. |
| `sirno upstream status` | Reports upstream lock, cache, glacier, and drift state. |

`sirno upstream add` accepts exactly one of `--branch NAME`, `--tag NAME`, or `--rev COMMIT`.
`--project PATH` selects a directory inside the Git tree that contains `Sirno.toml`.

## Check And Render

| Command | Behavior |
|---|---|
| `sirno check` | Checks the active lake. |
| `sirno render` | Creates or replaces Sirno-owned generated footer regions. |
| `sirno render -n, --dry` | Reports footer changes without writing files. |
| `sirno render --dry-run` | Alias for `sirno render --dry`. |
| `sirno render --override-json JSON` | Uses `[render.structural]` settings for that run only. |
| `sirno render delete` | Removes generated footer regions. |

The `sirno check -m, --mode` option selects the check boundary.
Render commands print changed paths or blocking diagnostics before their summary line.
Render commands operate on the active lake path.

The override JSON uses link relation names with edge direction lists,
such as `{"belongs":["to"]}`.
It does not write `Sirno.toml`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
