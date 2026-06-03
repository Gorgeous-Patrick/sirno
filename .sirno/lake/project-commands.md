---
name: Project Commands
desc: CLI commands for project setup, storage movement, Anchor, checks, and mist rendering.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

Project commands operate on the configured project, reservoir, Anchor baseline,
mist projection state, and generated footer state.

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

## Storage Movement

| Command | Behavior |
|---|---|
| `sirno lake move PATH` | Changes `[lake].path` and renames the reservoir directory. |
| `sirno lake mv PATH` | Short form of `sirno lake move PATH`. |
| `sirno move lake PATH` | Top-level wrapper for the same lake move. |
| `sirno mv lake PATH` | Short top-level wrapper for the same lake move. |

Path moves create missing destination parents and refuse existing destinations.
A destination inside the moved path is handled through temporary staging.

## Anchor

| Command | Behavior |
|---|---|
| `sirno anchor status` | Shows current reservoir ripples against `.sirno/anchor.toml`. |
| `sirno anchor check` | Validates `.sirno/anchor.toml` and compares it with the reservoir. |
| `sirno anchor update` | Accepts the current reservoir as the new Anchor baseline. |

Anchor update fails while open tide workitems remain.
Anchor update also fails while the default editable mist projection has pending ripples,
stale source fingerprints, missing projected entries, or staged misty-lake paths.
When `[tutorial]` is present,
this failure can include tutorial text controlled by `[tutorial].anchor_update_tide`
and `[tutorial].anchor_bootstrap_tide`.

Active review resolutions are stored in `.sirno/tide.toml`.
Anchor update deletes that file after accepting the waterline.

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
`--mist MIST` imports only entries selected by that mist in the upstream project.

## Check And Render

| Command | Behavior |
|---|---|
| `sirno check` | Checks the configured reservoir. |
| `sirno mist status [MIST]` | Reports pending mist ripples and stale projection state. |
| `sirno mist intake [MIST]` | Writes accepted misty-lake entry edits back into the reservoir. |
| `sirno mist render [MIST]` | Projects selected reservoir entries and renders generated navigation. |
| `sirno mist render -n, --dry` | Reports generated navigation changes without writing files. |
| `sirno mist render --dry-run` | Alias for `sirno mist render --dry`. |
| `sirno mist render --override-json JSON` | Uses temporary mist structural render settings for that run only. |
| `sirno mist render delete` | Removes generated navigation regions from a misty lake. |
| `sirno render ...` | Shorthand for `sirno mist render ...` on the default or active mist. |

The `sirno check -m, --mode` option selects the check boundary.
Mist status reports changed, stale, missing, and staged projection paths.
Mist intake accepts changed Markdown entries when the projection is fresh and editable.
Mist render forms print changed paths or blocking diagnostics before their summary line.
Mist render commands operate from the reservoir into the selected misty lake.

The override JSON uses link relation names with edge direction lists,
such as `{"belongs":["to"]}`.
It does not write the mist spec.
