---
desc: The generated lockfile used between raw and typed lake parsing.
name: Meta Registry
category:
  - concept
  - meta
belongs:
  - metadata
  - control-files
prerequisite:
  - meta-type
  - intrinsic
  - structural
---

The meta registry is the generated lockfile of meta-level entries discovered in a lake.

Sirno builds it during the first phase of lake loading.
That phase reads every entry file as raw Markdown frontmatter,
then records entries with `meta.type: "intrinsic"` or `meta.type: "structural"`.
It does not require typed intrinsic fields yet.

The second phase parses entries with the registry.
Discovered intrinsic fields become required plain-string fields.
Discovered structural fields become list-valued entry-address relations.

The project control copy lives at `.sirno/meta.toml`.
It is generated and tracked by Git.
Sirno rewrites it when the raw scan resolves to a different registry.
The lockfile makes meta-level resolution reviewable in ordinary diffs.
Authored design knowledge remains in the entries that define meta fields.

Registry order is entry-address order.
Rendered structural order belongs to the active mist settings.
