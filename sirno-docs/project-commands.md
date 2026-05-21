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

`sirno status` and `sirno st` summarize the configured project as an operational dashboard.
Status reports the config path, lake path, entry count, optional typed frost state,
structural field count, review-mode lake check, active tide summary,
and frost commit readiness.
CLI status keeps structural policy collapsed.
MCP status returns the full typed structural edge policy.

`sirno init` opens an interactive setup flow.
It asks which setup parts to run, asks for default paths when no path flag supplies them,
asks whether installed wrappers should be linked into Claude skills,
shows the init plan, and applies it after confirmation.

`sirno init --all` creates a Sirno config, ordinary seed entries,
an empty frost version `0`, and packaged skill wrappers without prompts.
By default, it names paths from the directory that contains `Sirno.toml`:
`<repo>-lake` for the lake path and `<repo>-frost` for the frost path.

`sirno init --lake PATH` chooses a non-default lake path.
`sirno init --frost PATH` chooses a non-default frost path.
`sirno init --no-lake`, `--no-frost`, and `--no-skills`
skip their corresponding setup parts.
`sirno init --claude-skills` creates `.claude/skills/sirno-*` links
to the installed `.agents/skills/sirno-*` package directories.
The config is still written when another selected setup part needs it.
When a setup part is skipped, its path option is not accepted.

`sirno lake init [PATH]` creates a Sirno config and ordinary seed entries without configuring frost.
`sirno lake move PATH` changes the configured lake path
and renames the current lake directory on the filesystem.
`sirno lake mv PATH` is its short form.
`sirno move lake PATH` and `sirno mv lake PATH` select the same path move.

`sirno frost init [PATH]` configures the frost path and records empty version `0`.
`sirno frost move PATH` changes the configured frost path
and renames the current frost path on the filesystem.
`sirno frost mv PATH` is its short form.
`sirno move frost PATH` and `sirno mv frost PATH` select the same path move.
Path moves create missing destination parents and refuse existing destinations.
A destination inside the moved path is handled through temporary staging.

`sirno commit` freezes the current lake
and writes the resulting current snapshot reference to `Sirno.lock.toml`.
`sirno frost commit` is its grouped form.
It fails while open tide workitems remain.
When `[tutorial]` is present,
this failure can include tutorial text controlled by `[tutorial].frost_commit_tide`
and `[tutorial].frost_bootstrap_tide`.
`sirno commit --unsafe-resolve-all` bypasses that gate for the current commit.

`sirno checkout --latest` materializes the latest version as a mutable lake.
`sirno defrost` is shorthand for `sirno checkout --latest`.
`sirno checkout VERSION` materializes one older version into the lake.
The grouped checkout command is `sirno frost checkout`.
The grouped latest shortcut is `sirno frost defrost`.
Version checkout is immutable unless `--unsafe-mutable` is supplied.
`sirno frost gc` garbage-collects private `eter` rows and artifact bytes
unreachable from the latest frost snapshot.
It updates the stored GC generation
and requires the lake to be the current mutable frostline.

`sirno upstream add DOMAIN --git SOURCE (--branch NAME | --tag NAME | --rev COMMIT)`
declares a Git-backed upstream lake and crystallizes it.
`--project PATH` selects a directory inside the Git tree that contains `Sirno.toml`.
`sirno upstream remove DOMAIN` removes the declaration and managed crystallized content.
`sirno upstream crystallize [DOMAIN] [--locked]` materializes upstreams into the lake.
`--locked` uses only existing lock records and cache mirrors.
`sirno upstream update [DOMAIN]` refreshes upstream locks and materialized content.
`sirno upstream status` reports missing locks, stale locks, cache misses,
missing crystallization,
and materialization drift.

`sirno check` checks the active lake.
The `-m, --mode` option selects the check boundary.

`sirno render` creates or replaces Sirno-owned generated footer regions.
`sirno render -n, --dry` reports generated footer regions that would change without writing files.
`--dry-run` is an alias for `--dry`.
Render commands print changed paths or blocking diagnostics before their summary line.

`sirno render --override-json JSON` uses JSON structural render settings for that run,
instead of the configured settings.
The JSON uses structural field and edge names,
such as `{"belongs":{"to":{"render":true}}}`.
It does not write `Sirno.toml`.
`sirno render delete` removes generated footer regions.
Render commands operate on the active lake path.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
