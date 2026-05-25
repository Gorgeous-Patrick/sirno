---
name: Project Commands
desc: CLI commands for project setup, storage movement, frost, checks, and rendering.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

Project commands operate on the configured project, active lake, optional frost path,
and generated footer state.

## Status

`sirno status` and `sirno st` summarize the configured project as an operational dashboard.
Status reports:

- config path;
- lake path;
- entry count;
- optional typed frost state;
- structural link relation count;
- review-mode lake check;
- active tide summary;
- frost commit readiness.

CLI status keeps structural policy collapsed.
MCP status returns the effective typed structural link policy,
including rendered directions from config and tide directions from relation entries.

## Setup

| Command | Behavior |
|---|---|
| `sirno init` | Opens an interactive setup flow. |
| `sirno init --all` | Runs full setup without prompts. |
| `sirno lake init [PATH]` | Creates config and seed entries without frost. |
| `sirno frost init [PATH]` | Configures frost and records empty version `0`. |

Interactive init asks which setup parts to run,
asks for default paths when no path flag supplies them,
asks whether installed wrappers should be linked into Claude skills,
shows the init plan,
and applies it after confirmation.

Full setup creates a Sirno config,
ordinary seed entries,
an empty frost version `0`,
and packaged skill wrappers.
Default paths are derived from the directory that contains `Sirno.toml`:

- `<repo>-lake` for the lake path;
- `<repo>-frost` for the frost path.

Setup flags:

| Flag | Behavior |
|---|---|
| `--lake PATH` | Chooses a non-default lake path. |
| `--frost PATH` | Chooses a non-default frost path. |
| `--no-lake` | Skips lake setup. |
| `--no-frost` | Skips frost setup. |
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
| `sirno frost move PATH` | Changes `[frost].path` and renames the frost path. |
| `sirno frost mv PATH` | Short form of `sirno frost move PATH`. |
| `sirno move frost PATH` | Top-level wrapper for the same frost move. |
| `sirno mv frost PATH` | Short top-level wrapper for the same frost move. |

Path moves create missing destination parents and refuse existing destinations.
A destination inside the moved path is handled through temporary staging.

## Frost

| Command | Behavior |
|---|---|
| `sirno commit` | Freezes the current lake and updates `Sirno.lock.toml`. |
| `sirno frost commit` | Grouped form of `sirno commit`. |
| `sirno commit --unsafe-resolve-all` | Bypasses open tide workitems for the current commit. |
| `sirno checkout --latest` | Materializes the latest version as a mutable lake. |
| `sirno defrost` | Shorthand for `sirno checkout --latest`. |
| `sirno checkout VERSION` | Materializes one older version into the lake. |
| `sirno frost checkout` | Grouped checkout command. |
| `sirno frost defrost` | Grouped latest shortcut. |
| `sirno frost gc` | Garbage-collects unreachable private frost data. |

Frost commit fails while open tide workitems remain.
When `[tutorial]` is present,
this failure can include tutorial text controlled by `[tutorial].frost_commit_tide`
and `[tutorial].frost_bootstrap_tide`.

Version checkout is immutable unless `--unsafe-mutable` is supplied.
Frost GC removes private `eter` rows and artifact bytes unreachable from the latest snapshot.
It updates the stored GC generation
and requires the lake to be the current mutable frostline.

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
| `sirno render --override-json JSON` | Uses structural render settings for that run only. |
| `sirno render delete` | Removes generated footer regions. |

The `sirno check -m, --mode` option selects the check boundary.
Render commands print changed paths or blocking diagnostics before their summary line.
Render commands operate on the active lake path.

The override JSON uses link relation and edge names,
such as `{"belongs":{"to":{"render":true}}}`.
It does not write `Sirno.toml`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
