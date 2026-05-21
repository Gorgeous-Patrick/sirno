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
frost,
and *upstream lakes*.
Generated config files include concise comments that describe how each written field is used.
`sirno util config` and `sirno util config tui` open an interactive terminal UI
for config section and comment maintenance.
It shows whether each top-level section is present
and whether that section's comments match Sirno's canonical renderer.
`sirno util config check` runs the non-interactive comment check without writing the file.
`sirno util config fix` runs the non-interactive rewrite through that renderer.

`[lake].path` names the lake path.
`[frost].path` optionally names the frost path.
`[upstreams.DOMAIN]` optionally declares one Git-backed upstream lake.
`[repo].members` optionally lists *repository* paths or globs scanned for *witness* blocks.
`[witness]` configures the delimiter regexes used to find *witness* blocks.
`[tutorial]` optionally enables extra CLI tutorial text for recoverable command failures.
Relative paths are resolved from the directory that contains `Sirno.toml`.
The CLI `--lake-path PATH` option can override `[lake].path` for one command.
The CLI `-F, --frost-path PATH` option selects a frost path for one direct frost check.

`[upstreams.DOMAIN]` declares an upstream lake crystallized under `DOMAIN`.
`DOMAIN` is an *entry atom* and becomes the injected *entry address* prefix.
It is an explicit local name with no default derived from `SOURCE`.
Each upstream has `git = "SOURCE"` and exactly one of `branch`, `tag`, or `rev`.
`SOURCE` is a remote Git URL or local Git repository source accepted by Git.
`project` optionally names the directory inside the Git tree that contains `Sirno.toml`;
it defaults to `.`.
There is no non-Git path upstream.
Every declared upstream is included by crystallization.
The domain shares the top-level lake namespace with implicit local sublakes,
so unmanaged files under `lake/DOMAIN/` block the declaration from being materialized.

A project can use Sirno without configured repo members or frost.
`sirno init` opens an interactive setup flow for the config, lake,
frost path, and packaged skill wrappers.
`sirno init --all` runs the full setup without prompts.
`sirno init --claude-skills` also links installed wrappers into `.claude/skills`.
Its default paths are derived from the directory that contains `Sirno.toml`:
`<repo>-lake` for `[lake].path` and `<repo>-frost` for `[frost].path`.
`sirno init --lake PATH` chooses a non-default lake path.
`sirno init --frost PATH` chooses a non-default frost path.
`sirno init --no-lake`, `--no-frost`, and `--no-skills`
skip their corresponding setup parts.
`--claude-skills` is available only when packaged skill wrappers are selected.
The config is still written when another selected setup part needs it.
When a setup part is skipped, its path option is not accepted.
`sirno lake init [PATH]` creates the config and lake without configuring frost.
`sirno lake move PATH` changes `[lake].path` and renames the lake directory.
`sirno lake mv PATH` is its short form.
`sirno frost init [PATH]` adds the frost config and records empty version `0`.
`sirno frost move PATH` changes `[frost].path` and renames the frost path.
`sirno frost mv PATH` is its short form.
`sirno move lake PATH` and `sirno move frost PATH`
select the same path moves from the top-level move group.
`sirno mv ...` is the short form for those wrappers.
Path moves create missing destination parents and refuse existing destinations.
A destination inside the moved path is handled through temporary staging.

`Sirno.lock.toml` records the lake's *frost* state when frost is configured
and the resolved upstream state when upstream lakes are configured.
It lives next to `Sirno.toml`.
The lock says whether the lake is current
or checked out to a frozen version.
It also pins each upstream to the exact Git commit crystallized into the lake.

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
`[check].structural-inhabitance` controls whether checks require each configured structural field
to name an existing *entry*.
When a check flag is present,
the config UI can restore that flag's canonical comment.
`sirno util config check` reports when the comment is missing.
`sirno util config fix` writes the comment.

`[tutorial]` controls optional instructional CLI output.
The table is absent by default.
When the table is present,
Sirno shows enabled tutorials after matching recoverable command failures.
`[tutorial].frost_commit_tide` explains a frost commit blocked by open *tide* workitems.
`[tutorial].frost_bootstrap_tide` adds first-snapshot context to that tutorial.
Removing the table silences all tutorial text.

`[structural]` controls which metadata fields are treated as structural.
Each structural field is written as a `[structural.FIELD]` subtable.
The field name should also name the *entry* that documents that structural field
and follow normal *entry atom* rules.
When `[check].structural-inhabitance` is enabled,
checks report configured structural fields without matching *entries*.
It must be a non-empty single-line metadata key,
must not contain a comma,
and must not be `name`, `desc`, or `frozen`.
The subtable may define `to`, `from`, and `clique` edge policies.
This repository recommends `category`, `belongs`, `prerequisite`, and `refines`.
The key order is user-authored project structure.
Sirno preserves that order when it rewrites `Sirno.toml`.
Each edge policy may set `render = true`
and `ripple = { lake = bool, frost = bool }`.
Absent values are false.

`to` links from the *entry* to metadata targets.
`from` links from the *entry* to *entries* that name it as a metadata target.
`clique` adds separate clique-derived sections through shared targets in that field.

`render` controls generated footer output.
`ripple.lake` and `ripple.frost` control which edge directions produce *tide* workitems.

`Sirno.lock.toml` also records explicit *tide* resolutions when frost is configured.
Those resolutions are compared against the current ripple fingerprint.
They are cleared after a successful frost commit.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [project-config-comments](project-config-comments.md)

> **Sirno generated links end.**
