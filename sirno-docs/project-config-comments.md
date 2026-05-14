---
name: Project Config Comments
description: The exact comments Sirno writes beside generated project config fields.
category:
  - concept
clustee:
  - project-config
refiner:
  - project-config
witness:
---

Project config comments are the field-level comments Sirno writes when it renders `Sirno.toml`.

Each comment sits immediately above the field it describes.
Optional table comments appear only when the optional table is written.
Parsing still ignores comments through ordinary TOML rules.

The generated comments are:

- `Markdown monograph path, resolved relative to this config file.`
- `Markdown entry store path, resolved relative to this config file.`
- `Store-root paths Sirno skips while reading, checking, querying, and generating links.`
- `Private eter history root, kept outside the public store.`
- `Repository files, directories, or globs scanned for witness blocks.`
- `Require generated footers to match current metadata during checks.`
- `Include category links; use a boolean or { to = bool, from = bool }.`
- `Include clustee links; use a boolean or { to = bool, from = bool }.`
- `Add clique sections derived from clustee closures.`
- `Include refiner links; use a boolean or { to = bool, from = bool }.`

The comments explain use, not schema authority.
The Rust config types and TOML parser remain the schema boundary.

---

> **Sirno generated links begin. Do not edit this section.**

Clustee (to):
- [project-config](project-config.md)

> **Sirno generated links end.**
