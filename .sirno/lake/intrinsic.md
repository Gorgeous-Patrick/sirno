---
name: Intrinsic
desc: The meta.type value for entries that define required intrinsic metadata fields.
category:
  - concept
  - meta
belongs:
  - meta-type
prerequisite:
  - meta-type
---

`meta.type: "intrinsic"` is the `meta.type` value for intrinsic metadata fields.
It marks an *entry* as the definition of one required field.

Sirno discovers intrinsic fields during the raw meta-registry scan.
Each discovered intrinsic field is required on every typed *entry*,
and its value is a plain string.

This lake currently defines `name` and `desc` as intrinsic fields.
They are ordinary entries,
so their meaning can be read, queried, reviewed, and versioned beside project-specific entries.

Intrinsic field names use the same metadata-key validity rules as other meta-level fields.
Sirno reports invalid intrinsic field names during checks.
