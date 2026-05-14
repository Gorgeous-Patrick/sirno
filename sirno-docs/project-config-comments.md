---
name: Project Config Comments
description: The exact comments Sirno writes beside generated project config fields.
category:
  - concept
refines:
  - project-config
---

Project config comments are the field-level comments Sirno writes when it renders `Sirno.toml`.

Each comment sits immediately above the field it describes.
Optional table comments appear only when the optional table is written.
Parsing still ignores comments through ordinary TOML rules.

The generated comments are:

- `Markdown monograph path, resolved relative to this config file.`
- `Markdown entry lake path, resolved relative to this config file.`
- `Lake-root paths Sirno skips while reading, checking, querying, and generating links.`
- `Sirno Frost root, kept outside the public lake.`
- `Repository files, directories, or globs scanned for witness blocks.`
- `Require generated footers to match current metadata during checks.`
- `Include category links; use a boolean or { to = bool, from = bool }.`
- `Include belongs links; use a boolean or { to = bool, from = bool }.`
- `Add clique sections derived from belongs targets.`
- `Include refines links; use a boolean or { to = bool, from = bool }.`

The comments explain use, not schema authority.
The Rust config types and TOML parser remain the schema boundary.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to): (none)

> **Sirno generated links end.**
