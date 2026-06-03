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
it tells Sirno that the entry defines a schema-level role rather than only project prose.

The current values are documented by their role entries:

| Value | Role entry | Meaning |
|---|---|---|
| `intrinsic` | `intrinsic` | The entry defines a required built-in metadata field. |
| `structural` | `structural` | The entry defines a configured structural link relation. |

The role entries document the allowed carriers for each marker.
They keep the compact metadata table readable while giving each marker a stable review target.
