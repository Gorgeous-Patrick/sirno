---
name: Future Work
description: Reserved design areas that may be refined later.
category:
  - concept
clustee:
  - sirno
---

Several design areas are reserved for later refinement.

The `locked` field may later define how entries or generated regions resist accidental edits.

`eter` versioning is reserved for later design.
The current design depends only on durable storage and indexing.

The direction names may still be refined.
The current names are `lower`, `raise`, `realize`, and `reflect`.

Planning skills are future work.
They may use entries to leave durable work artifacts without changing Sirno's core fields.

Future work should remain explicit without becoming speculative architecture.
The current design is useful because its core is small:
entries, metadata, structural fields, generated footers, surfaces, directions, checks, and witnesses.
New features should preserve that clarity.

The `locked` field is one example.
It may eventually protect entries,
metadata fields,
or generated regions that a project treats as controlled.
That design needs a clear ownership model before it becomes part of the schema.
Until then, leaving the field reserved is safer than accepting vague lock behavior.

Versioning is another example.
`eter` may later provide history, snapshots, branching, or reviewable store states.
Sirno should adopt only the versioning concepts that help design work move between surfaces.
It should not make the public entry model harder to read just to expose storage internals.

Direction names may also evolve.
The current names are compact and memorable,
but they should remain subordinate to the model they describe.
If the project learns a clearer vocabulary,
entries and manuals can reflect that deliberately.

---

> **Sirno generated links begin. Do not edit this section.**

- [sirno](sirno.md)

> **Sirno generated links end.**
