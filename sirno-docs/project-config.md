---
name: Project Config
description: The Sirno.toml file that marks and configures a Sirno-managed repository.
category:
  - concept
belongs:
  - sirno-lake
refines:
  - storage-and-interfaces
---

`Sirno.toml` marks a repository as Sirno-managed.

The file configures the public entry lake
and the operational policies that Sirno applies to the lake.
It may also configure a monograph,
repository witness members,
and Sirno Frost.
Generated config files include concise comments that describe how each written field is used.

`[mono].path` optionally names the monograph.
`[lake].path` names the Markdown entry lake.
`[frost].path` optionally names the private Sirno Frost root.
`[repo].members` optionally lists repository paths or globs scanned for witness blocks.
Relative paths are resolved from the directory that contains `Sirno.toml`.

A project can use Sirno without a configured monograph, repo members, or Sirno Frost.
`sirno init` creates the config and public entry lake.
`sirno mv PATH` changes `[lake].path` and renames the public lake directory.
`sirno frost init` adds the Frost config and freezes the current public lake.
`sirno frost mv PATH` changes `[frost].path` and renames the private Frost root.

`Sirno.lock` records the public lake's Frost state when Sirno Frost is configured.
It lives next to `Sirno.toml`.
The lock says whether the lake is current
or checked out to a frozen version.

`[lake].ignore` lists paths relative to the lake root.
Sirno skips those paths and their descendants while reading, checking,
querying, and changing generated links.
Ignored paths are for adjacent tool state, not for entries.

`[repo].members` lists paths and globs relative to `Sirno.toml` when repo witnesses are enabled.
File members are scanned directly.
Directory members are scanned recursively.
Glob members may match files or directories.

`[check].link` controls generated-link freshness checks.
It is enabled by default.
Malformed generated-link sentinels remain errors,
because malformed sentinels make Sirno ownership ambiguous.

`[links]` controls which structural fields are projected into generated footers.
`category`, `belongs`, and `refines` each accept either a boolean
or `{ to = boolean, from = boolean }`.
A boolean applies to both link sides.

`to` links from the entry to metadata targets.
`from` links from the entry to entries that name it as a metadata target.
`links.clique` adds separate clique-derived sections through named `belongs` targets.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to):
- [sirno-lake](sirno-lake.md)

> **Sirno generated links end.**
