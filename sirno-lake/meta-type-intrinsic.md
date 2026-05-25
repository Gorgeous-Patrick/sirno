---
name: Intrinsic Metadata Type
desc: The meta.type value for entries that define required built-in metadata fields.
category:
  - concept
  - meta
belongs:
  - meta-type
prerequisite:
  - meta-type
---

`meta.type: "intrinsic"` is the `meta.type` value for built-in metadata fields.
It marks an *entry* as the definition of one required field.

The valid intrinsic field entries are `name` and `desc`.
Those fields are required on every *entry*,
and their values are plain strings.

The marker keeps Sirno's built-in entry shape self-described in the lake.
The fields are still ordinary entries,
so their meaning can be read, queried, reviewed, and versioned beside project-specific entries.

Only `name` and `desc` may carry this marker.
If another entry carries `meta.type: "intrinsic"`,
Sirno reports that mismatch during checks.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [meta-type](meta-type.md)
- belongs (from):
  - [desc](desc.md)
  - [name](name.md)

> **Sirno generated links end.**
