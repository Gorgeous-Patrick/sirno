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
- `frost path, kept outside the lake.`
- `Git-backed upstream lake crystallized into a glacier under this entry domain.`
- `Repository files, directories, or globs scanned for witness blocks.`
- `Witness delimiter regex pairs; each first capture group is the entry address.`
- `Canonical entry-address capture: ([^\x00-\x1F\x7F<>:"/\\|?*,\r\n]+)`
- `Require generated footers to match current metadata during checks.`
- `Require each configured link relation to have a matching structural relation entry during checks.`
- `Presence of this table enables tutorial text for recoverable command failures.`
- `Remove this table to keep CLI errors terse.`
- `Show tutorial text when frost commit is blocked by open tide workitems.`
- `Include first-snapshot bootstrap context in the frost commit tide tutorial.`
- `Structural link relations.`
- `Add one [structural.FIELD] subtable for each metadata relation Sirno treats as structure.`
- `FIELD must name the lake entry that documents the relation and follow normal entry-atom rules.`
- `FIELD must be a non-empty single-line metadata key with no comma.`
- `FIELD cannot be name, desc, meta, or frozen.`
- `Entry metadata values for FIELD must be lists of entry addresses; targets must exist by review.`
- `` `to` follows outgoing targets, `from` incoming sources, and `clique` shared-target neighbors. ``
- `render = true writes generated footer links.`
- `Tide policy lives in structural relation entry meta.ripple.lake and meta.ripple.frost direction lists.`
- `Omitted render values are false.`

This list is the canonical source for the strings.
The Rust config renderer materializes it,
and `sirno util config check` and `sirno util config fix` keep an active `Sirno.toml` aligned.
A *witness* block on the renderer should bind the implementation back to this *entry*
so the lake and the renderer stay in sync.

The comments explain use, not schema authority.
The Rust config types and TOML parser remain the schema boundary.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [project-config](project-config.md)
- belongs (from): (none)

> **Sirno generated links end.**
