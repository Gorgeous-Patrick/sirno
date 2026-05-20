---
desc: The exact comments Sirno writes beside generated project config fields.
lifecycle: Active
name: Project Config Comments
structural:
  category:
  - concept
  refines:
  - project-config
---

Project config comments are the field-level comments Sirno writes when it renders `Sirno.toml`.

Each comment sits immediately above the field or repeated field group it describes.
Optional table comments appear only when the optional table is written.
Parsing still ignores comments through ordinary TOML rules.

`sirno util config` compares the active config file with the comment set
that the canonical renderer would write for the parsed config.
It reports missing comments without changing the file.
`sirno util config --fix` rewrites the file through that renderer when comments are missing.

The generated comments are:

- `Markdown entry lake path, resolved relative to this config file.`
- `Paths in lake that Sirno skips while reading, checking, querying, and rendering footers.`
- `Sirno Frost path, kept outside the public lake.`
- `Repository files, directories, or globs scanned for witness blocks.`
- `Witness delimiter regex pairs; each first capture group is the entry id.`
- `Canonical filename entry-id capture: ([^\x00-\x1F\x7F<>:"/\\|?*.,\r\n]+)`
- `Require generated footers to match current metadata during checks.`
- `Presence of this table enables tutorial text for recoverable command failures.`
- `Remove this table to keep CLI errors terse.`
- `Show tutorial text when frost commit is blocked by open tide workitems.`
- `Include first-snapshot bootstrap context in the frost commit tide tutorial.`
- `Structural metadata fields.`
- `Add one [structural.FIELD] subtable for each metadata field Sirno treats as structure.`
- `FIELD must name the lake entry that documents the field and follow normal entry-id rules.`
- `FIELD must be a non-empty single-line metadata key with no comma.`
- `FIELD cannot be name, desc, or frozen.`
- `Entry metadata values for FIELD must be lists of entry ids; targets must exist by review.`
- `` `to` follows outgoing targets, `from` incoming sources, and `clique` shared-target neighbors. ``
- `render = true writes generated footer links.`
- `ripple.lake and ripple.frost add tide workitems from the waterline and frostline.`
- `Omitted render and ripple values are false.`

The comments explain use, not schema authority.
The Rust config types and TOML parser remain the schema boundary.
