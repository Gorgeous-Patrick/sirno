---
name: Project Config Comments
desc: The exact comments Sirno writes beside generated project config fields.
category:
  - concept
refines:
  - project-config
---

Project config comments are the field-level comments Sirno writes when it renders `Sirno.toml`.

Each comment sits immediately above the field or repeated field group it describes.
Optional table comments appear only when the optional table is written.
Parsing still ignores comments through ordinary TOML rules.

The generated comments are:

- `Markdown monograph path, resolved relative to this config file.`
- `Markdown entry lake path, resolved relative to this config file.`
- `Paths in lake that Sirno skips while reading, checking, querying, and rendering footers.`
- `Sirno Frost path, kept outside the public lake.`
- `Repository files, directories, or globs scanned for witness blocks.`
- `Witness delimiter regex pairs; each first capture group is the entry id.`
- `Canonical filename entry-id capture: ([^\x00-\x1F\x7F<>:"/\\|?*,\r\n]+)`
- `Require generated footers to match current metadata during checks.`
- `Structural metadata fields; render and ripple settings default to false.`

The comments explain use, not schema authority.
The Rust config types and TOML parser remain the schema boundary.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from): (none)

> **Sirno generated links end.**
