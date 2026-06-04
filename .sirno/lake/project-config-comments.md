---
name: Project Config Comments
desc: The exact comments Sirno writes beside generated project config fields.
category:
  - concept
belongs:
  - project-config
prerequisite:
  - project-config
---

Project config comments are the field-level comments Sirno writes when it renders `Sirno.toml`.

Each comment sits immediately above the field or repeated field group it describes.
Optional table comments appear only when the optional table is written.
Check flag comments appear only when the corresponding check flag is written.
Parsing still ignores comments through ordinary TOML rules.

The canonical comments are:

- `Sirno Lake path, resolved relative to this config file.`
- `Paths in lake that Sirno skips while reading, checking, querying, and rendering footers.`
- `Git-backed upstream lake crystallized into a glacier under this entry domain.`
- `Optional manifest selects the upstream project config file.`
- `Optional mist selects the imported portion from the upstream project.`
- `Repository files, directories, or globs scanned for witness blocks.`
- `Witness delimiter regex pairs; each first capture group is the entry address.`
- `Canonical entry-address capture: ([^\x00-\x1F\x7F<>:"/\\|?*,\r\n]+)`
- `Require generated footers to match current metadata during checks.`
- `Presence of this table enables tutorial text for recoverable command failures.`
- `Remove this table to keep CLI errors terse.`
- `Show tutorial text when anchor update is blocked by open tide workitems.`
- `Include first-anchor bootstrap context in the anchor update tide tutorial.`
- `Generated-footer structural link render policy.`
- `Each key names a discovered structural relation.`
- `Values are direction lists: to, from, and clique.`

This list is the canonical source for the strings.
The Rust config renderer materializes it,
and `sirno util config check` and `sirno util config fix` keep an active `Sirno.toml` aligned.
A *witness* block on the renderer should bind the implementation back to this *entry*
so the lake and the renderer stay in sync.

The comments explain use, not schema authority.
The Rust config types and TOML parser remain the schema boundary.
Mist specs own generated-footer structural render comments.
The `Sirno.toml` renderer does not write render comments.
