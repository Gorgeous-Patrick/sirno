---
name: Project Config
desc: The Sirno.toml file that marks and configures a Sirno-managed repository.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - storage
---

`Sirno.toml` marks a *repository* as Sirno-managed.

The file configures the lake
and the operational policies that Sirno applies to the *lake*.
It may also configure *repository witness* members
and *upstream lakes*.
Generated config files include concise comments that describe how each written field is used.
`sirno util config` and `sirno util config tui` open an interactive terminal UI
for config section and comment maintenance.
It shows whether each top-level section is present
and whether that section's comments match Sirno's canonical renderer.
`sirno util config check` runs the non-interactive comment check without writing the file.
`sirno util config fix` runs the non-interactive rewrite through that renderer.

The main fields are:

| Field | Meaning |
|---|---|
| `[lake].path` | Names the lake path. |
| `[upstreams.DOMAIN]` | Optionally declares one Git-backed upstream lake. |
| `[repo].members` | Lists *repository* paths or globs scanned for *witness* blocks. |
| `[witness]` | Configures the delimiter regexes used to find *witness* blocks. |
| `[tutorial]` | Enables extra CLI tutorial text for recoverable command failures. |

Path selection follows three rules:

- Relative paths are resolved from the directory that contains `Sirno.toml`.
- The CLI `--lake-path PATH` option can override `[lake].path` for one command.
- Sirno control files live in `.sirno/` next to `Sirno.toml`.

`[upstreams.DOMAIN]` declares an upstream lake crystallized into a glacier under `DOMAIN`.
`DOMAIN` is an *entry atom* and becomes the glacier *entry address* prefix.
It is an explicit local name with no default derived from `SOURCE`.
Each upstream has `git = "SOURCE"` and exactly one of `branch`, `tag`, or `rev`.
`SOURCE` is a remote Git URL or local Git repository source accepted by Git.
`project` optionally names the directory inside the Git tree that contains `Sirno.toml`;
it defaults to `.`.
There is no non-Git path upstream.
Every declared upstream is included by crystallization.
The glacier domain shares its lake path with implicit local lakelets,
so unmanaged files under `lake/DOMAIN/` block the declaration from being crystallized.

The Anchor baseline is not configured in `Sirno.toml`.
It lives at `.sirno/anchor.toml`.
`sirno anchor update` creates or replaces it after the lake passes review.

A project can use Sirno without configured repo members or upstreams.
`sirno init` opens an interactive setup flow for the config, lake,
and packaged skill wrappers.
`sirno init --all` runs the full setup without prompts.
`sirno init --claude-skills` also links installed wrappers into `.claude/skills`.
Its default paths are derived from the directory that contains `Sirno.toml`:
`<repo>-lake` for `[lake].path`.
`sirno init --lake PATH` chooses a non-default lake path.
`sirno init --no-lake` and `--no-skills`
skip their corresponding setup parts.
`--claude-skills` is available only when packaged skill wrappers are selected.
The config is still written when another selected setup part needs it.
When a setup part is skipped, its path option is not accepted.
`sirno lake init [PATH]` creates the config and lake.
`sirno lake move PATH` changes `[lake].path` and renames the lake directory.
`sirno lake mv PATH` is its short form.
`sirno move lake PATH` selects the same path move from the top-level move group.
`sirno mv ...` is the short form for those wrappers.
Path moves create missing destination parents and refuse existing destinations.
A destination inside the moved path is handled through temporary staging.

`Sirno.lock.toml` records the resolved upstream state when upstream lakes are configured.
It lives next to `Sirno.toml`.
It also pins each upstream to the exact Git commit crystallized into the glacier.
Anchor state belongs in `.sirno/anchor.toml`.
Temporary Tide resolutions may still use `Sirno.lock.toml` until `.sirno/tide.toml` is actualized.

`[lake].ignore` lists paths relative to the *lake* root.
Sirno skips those paths and their descendants while reading, checking,
querying, and rendering generated footers.
Ignored paths are for adjacent tool state, not for *entries*.

`[repo].members` lists paths and globs relative to `Sirno.toml` when repo *witnesses* are enabled.
File members are scanned directly.
Directory members are scanned recursively.
Glob members may match files or directories.

`[[witness.delimiters]]` configures one *witness* delimiter syntax.
Each delimiter table has `begin` and `end` regex fields.
Each regex should capture the *entry address* as its first capture group.
Sirno rejects empty, invalid, captureless, or empty-matching delimiter regexes.
An empty delimiter list disables repository witness lookup.
Generated configs write the standard syntax,
which accepts `//` line comments and hidden Markdown HTML comments.
The standard regexes use one canonical capture for valid *entry addresses*.
Configured regexes may be narrower,
but they should include every *entry address* allowed by the active project policy.

`[check]` is optional.
Omitting the table or an individual check flag leaves that check enabled.
`[check].render` controls generated-footer freshness checks.
Malformed generated-link sentinels remain errors,
because malformed sentinels make Sirno ownership ambiguous.
`[check].structural-inhabitance` controls whether checks require each configured link relation
to name an existing *entry* with `meta.type: "structural"`.
When a check flag is present,
the config UI can restore that flag's canonical comment.
`sirno util config check` reports when the comment is missing.
`sirno util config fix` writes the comment.

`[tutorial]` controls optional instructional CLI output.
The table is absent by default.
When the table is present,
Sirno shows enabled tutorials after matching recoverable command failures.
`[tutorial].anchor_update_tide` explains an anchor update blocked by open *tide* workitems.
`[tutorial].anchor_bootstrap_tide` adds first-anchor context to that tutorial.
Removing the table silences all tutorial text.

`[structural]` controls which metadata fields define structural link relations.
Each link relation is written as a `[structural.FIELD]` subtable.

| Part | Rule |
|---|---|
| `FIELD` | Names the metadata relation. |
| field shape | Uses a non-empty single-line metadata key. |
| reserved names | `FIELD` must not contain a comma or be `name`, `desc`, or `frozen`. |
| `entry` | Names the *entry* that documents the relation. |
| relation entry | Checks can require that *entry* to define `meta.type: "structural"`. |
| key order | Records user-authored project structure and is preserved by rewrites. |

This repository recommends `category`, `belongs`, `prerequisite`, and `refines`.
When `[check].structural-inhabitance` is enabled,
checks report configured relation entries that are missing or not structural.

`[render.structural]` controls generated footer output.
Each key names a configured structural relation.
Each value is a list of rendered edge directions.
Absent relations render no generated footer groups.

| Edge | Generated relation |
|---|---|
| `to` | Links from the *entry* to metadata targets. |
| `from` | Links from the *entry* to *entries* that name it as a metadata target. |
| `clique` | Adds separate sections through shared targets in that relation. |

Tide policy lives in structural relation entry `meta.ripple.lake` and `meta.ripple.anchor` direction lists.

`Sirno.lock.toml` temporarily records explicit *tide* resolutions.
Those resolutions are compared against the current ripple fingerprint.
They are cleared after a successful anchor update.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [project-config-comments](project-config-comments.md)

> **Sirno generated links end.**
