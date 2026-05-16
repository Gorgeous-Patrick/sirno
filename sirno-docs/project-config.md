---
name: Project Config
desc: The Sirno.toml file that marks and configures a Sirno-managed repository.
category:
  - concept
belongs:
  - sirno-lake
refines:
  - storage
---

`Sirno.toml` marks a *repository* as Sirno-managed.

The file configures the public *entry lake*
and the operational policies that Sirno applies to the *lake*.
It may also configure a *monograph*,
*repository witness* members,
and Sirno Frost.
Generated config files include concise comments that describe how each written field is used.

`[mono].path` optionally names the *monograph*.
`[lake].path` names the Markdown *entry lake*.
`[frost].path` optionally names the private Sirno Frost path.
`[repo].members` optionally lists *repository* paths or globs scanned for *witness* blocks.
`[witness]` configures the delimiter regexes used to find *witness* blocks.
Relative paths are resolved from the directory that contains `Sirno.toml`.
The CLI `--lake-path PATH` option can override `[lake].path` for one command.

A project can use Sirno without a configured *monograph*, repo members, or Sirno Frost.
`sirno init` creates the config and public *entry lake*.
`sirno move PATH` changes `[lake].path` and renames the public *lake* directory.
`sirno mv PATH` is its short form.
`sirno frost init` adds the Sirno Frost config and records empty version `0`.
`sirno frost move PATH` changes `[frost].path` and renames the private *frost* path.
`sirno frost mv PATH` is its short form.

`Sirno.lock.toml` records the public *lake*'s *frost* state when Sirno Frost is configured.
It lives next to `Sirno.toml`.
The lock says whether the *lake* is current
or checked out to a frozen version.

`[lake].ignore` lists paths relative to the *lake* root.
Sirno skips those paths and their descendants while reading, checking,
querying, and changing generated links.
Ignored paths are for adjacent tool state, not for *entries*.

`[repo].members` lists paths and globs relative to `Sirno.toml` when repo *witnesses* are enabled.
File members are scanned directly.
Directory members are scanned recursively.
Glob members may match files or directories.

`[[witness.delimiters]]` configures one *witness* delimiter syntax.
Each delimiter table has `begin` and `end` regex fields.
Each regex should capture the *entry* id as its first capture group.
Sirno rejects empty, invalid, captureless, or empty-matching delimiter regexes.
At least one delimiter table is required so the *repository* syntax is explicit.
Generated configs write the standard syntax,
which accepts `//` line comments and hidden Markdown HTML comments.
The standard regexes use one canonical capture for filename-like *entry* ids.
Configured regexes may be narrower,
but they should include every *entry* id allowed by the active project policy.

`[check].link` controls generated-link freshness checks.
It is enabled by default.
Malformed generated-link sentinels remain errors,
because malformed sentinels make Sirno ownership ambiguous.

`[structural]` controls which metadata fields are treated as structural.
Each field key maps to a table with `link = { to = bool, from = bool, clique = bool }`.
This repository recommends `category`, `belongs`, and `refines`.
The key order is user-authored project structure.
Sirno preserves that order when it rewrites `Sirno.toml`.
Each `link` boolean is optional,
and an absent boolean means false.

`to` links from the *entry* to metadata targets.
`from` links from the *entry* to *entries* that name it as a metadata target.
`clique` adds separate clique-derived sections through shared targets in that field.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
