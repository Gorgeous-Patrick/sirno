---
name: Meta Type
desc: The metadata discriminator that marks entries with built-in Sirno schema roles.
category:
  - concept
  - meta
belongs:
  - metadata
prerequisite:
  - metadata
---

`meta.type` is a Sirno-managed metadata discriminator for entries with built-in schema roles.

It is optional for ordinary entries.
When present,
the raw meta-registry scan treats the entry as a schema-level definition.

The current values are documented by their role entries:

| Value | Role entry | Meaning |
|---|---|---|
| `intrinsic` | `intrinsic` | The entry defines a required intrinsic metadata field. |
| `structural` | `structural` | The entry defines a structural link relation. |

The role entries document the allowed carriers for each marker.
They keep the compact metadata table readable while giving each marker a stable review target.
The generated `meta-registry` records the discovered carriers for typed parsing.
