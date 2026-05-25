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

The current values are:

| Value | Meaning |
|---|---|
| `intrinsic` | The entry defines a required built-in metadata field. |
| `structural` | The entry defines a configured structural link relation. |

The value entries document the allowed carriers for each marker.
They keep the compact metadata table readable while giving each role a stable review target.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [metadata](metadata.md)
- belongs (from):
  - [meta-type-intrinsic](meta-type-intrinsic.md)
  - [meta-type-structural](meta-type-structural.md)

> **Sirno generated links end.**
