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

The file configures the reservoir
and the project-level operational policies that Sirno applies to the lake.
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
| `[lake].path` | Names the reservoir path. |
| `[upstreams.DOMAIN]` | Optionally declares one Git-backed upstream lake. |
| `[repo].members` | Lists *repository* paths or globs scanned for *witness* blocks. |
| `[witness]` | Configures the delimiter regexes used to find *witness* blocks. |
| `[tutorial]` | Enables extra CLI tutorial text for recoverable command failures. |

Path selection follows three rules:

- Relative paths are resolved from the directory that contains `Sirno.toml`.
- The CLI `--lake-path PATH` option can override the reservoir path for one command.
- Sirno control files live in `.sirno/` next to `Sirno.toml`.

`[upstreams.DOMAIN]` declares an upstream lake crystallized into a glacier under `DOMAIN`.
`DOMAIN` is an *entry atom* and becomes the glacier *entry address* prefix.
It is an explicit local name with no default derived from `SOURCE`.
Each upstream has `git = "SOURCE"` and exactly one of `branch`, `tag`, or `rev`.
`SOURCE` is a remote Git URL or local Git repository source accepted by Git.
`project` optionally names the upstream project root inside the Git tree;
it defaults to `.`.
`manifest` optionally names the project config manifest relative to `project`;
it defaults to `Sirno.toml`.
The manifest path may be nested and may use another filename.
There is no non-Git path upstream.
Every declared upstream is included by crystallization.
The glacier domain shares its reservoir path with implicit local lakelets,
so unmanaged files under `lake/DOMAIN/` block the declaration from being crystallized.

The Anchor baseline is not configured in `Sirno.toml`.
It lives at `.sirno/anchor.toml`.
`sirno anchor update` creates or replaces it after the lake passes review.
Other Sirno-owned control state belongs under `.sirno/`.

A project can use Sirno without configured repo members or upstreams.
`sirno init` opens an interactive setup flow for the config, lake,
and packaged skill wrappers.
`sirno init --all` runs the full setup without prompts.
`sirno init --claude-skills` also links installed wrappers into `.claude/skills`.
The default reservoir path is `.sirno/lake` next to `Sirno.toml`.
The default misty workspace renders to `sirno-lake/`.
`sirno init --lake PATH` chooses a non-default reservoir path.
`sirno init --no-lake` and `--no-skills`
skip their corresponding setup parts.
`--claude-skills` is available only when packaged skill wrappers are selected.
The config is still written when another selected setup part needs it.
When a setup part is skipped, its path option is not accepted.
`sirno lake init [PATH]` creates the config and reservoir.
The `lake-commands` entry owns lake initialization, checking, and movement command behavior.

`.sirno/upstream.toml` records the resolved upstream state when upstream lakes are configured.
It pins each upstream to the exact Git commit crystallized into the glacier.
Anchor state belongs in `.sirno/anchor.toml`.
Active Tide resolutions belong in `.sirno/tide.toml`.
The generated meta registry belongs in `.sirno/meta.toml`.
It is a tracked lockfile.
Sirno rewrites it when the raw meta scan changes.

`[lake].ignore` lists paths relative to the reservoir root.
Sirno skips those paths and their descendants while reading, checking,
and querying reservoir entries.
Projection settings also preserve `.sirno/` inside misty lakes.
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
`[check].render` controls generated-footer freshness checks for checked entry directories.
Malformed generated-link sentinels remain errors,
because malformed sentinels make Sirno ownership ambiguous.
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

`Sirno.toml` does not store intrinsic or structural field registration.
Sirno discovers both from lake entries that define `meta.type`.
An intrinsic or structural entry address is the metadata field name.
Structural relation order is entry-address order.

Generated footer structural rendering belongs to mist settings,
not to project configuration.
A mist spec owns `[render.structural]` for that projection.
Each key names a discovered structural relation.
Each value is a list of rendered edge directions.
Absent relations render no generated footer groups in that misty lake.

| Edge | Generated relation |
|---|---|
| `to` | Links from the *entry* to metadata targets. |
| `from` | Links from the *entry* to *entries* that name it as a metadata target. |
| `clique` | Adds separate sections through shared targets in that relation. |

Tide policy lives in structural relation entry `meta.ripple.lake` and `meta.ripple.anchor` direction lists.

`.sirno/tide.toml` records explicit *tide* resolutions.
Those resolutions are compared against the current ripple fingerprint.
They are cleared after a successful anchor update.
