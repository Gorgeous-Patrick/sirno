---
name: Project Config
description: The Sirno.toml file that marks and configures a Sirno-managed repository.
category:
  - concept
clustee:
  - sirno-store
refiner:
  - storage-and-interfaces
---

`Sirno.toml` marks a repository as Sirno-managed.

The file configures the monograph, the public entry store,
and the operational policies that Sirno applies to that store.

`[mono].path` names the monograph.
`[store].path` names the Markdown entry store.
Relative paths are resolved from the directory that contains `Sirno.toml`.

`[store].ignore` lists paths relative to the store root.
Sirno skips those paths and their descendants while reading, checking,
querying, and changing generated links.
Ignored paths are for adjacent tool state, not for entries.

`[check].link` controls generated-link freshness checks.
It is enabled by default.
Malformed generated-link sentinels remain errors,
because malformed sentinels make Sirno ownership ambiguous.

`[links]` controls which structural fields are projected into generated footers.
`category`, `clustee`, and `refiner` each accept either a boolean
or `{ to = boolean, from = boolean }`.
A boolean applies to both directions.

`to` links from the entry to metadata targets.
`from` links from the entry to entries that name it as a metadata target.
`links.clique` adds separate clique-derived sections through named clustee closures.

---

> **Sirno generated links begin. Do not edit this section.**

Category (from): (none)

Category (to)
- [concept](concept.md)

Clique
- [entry](entry.md)
- [generated-footer](generated-footer.md)
- [metadata](metadata.md)
- [query](query.md)
- [sirno-store](sirno-store.md)
- [structural-check](structural-check.md)

> **Sirno generated links end.**
