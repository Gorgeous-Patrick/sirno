---
name: Future Work
desc: Reserved design areas that may be refined later.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - sirno
---

Several design areas are reserved for later refinement.

The `locked` field may later define how *entries* or generated regions resist accidental edits.

Long-term version retention policy is reserved for later design.
The core model already treats versions as `eter` snapshots.
Future work decides which snapshots Sirno keeps by default,
which snapshots can be named,
and how review interfaces expose them.

The *transform* names are now `actualize` and `internalize`.
Future work may add command support for those names where the CLI still exposes older surfaces.

Lake dependency management now has initial vocabulary:
*entry domains*, *entry addresses*, and *lake sheaves*.
Future work should define resolution rules, symlink materialization,
and version selection without making entry naming carry all dependency policy.

Future work should remain explicit without becoming speculative architecture.
The current design is useful because its core is small:
*entries*, metadata, structural fields, generated footers, forms, *transforms*, checks, and *witnesses*.
New features should preserve that clarity.

The `locked` field is one example.
It may eventually protect *entries*,
metadata fields,
or generated regions that a project treats as controlled.
That design needs a clear ownership model before it becomes part of the schema.
Until then, leaving the field reserved is safer than accepting vague lock behavior.

Version retention is another example.
`eter` provides history, snapshots, retirement, and garbage collection.
`sirno frost gc` provides a latest-snapshot collection policy for frost rows and artifact bytes.
Sirno still needs long-term policy for which historical snapshots stay live.
That policy should preserve reviewable *lake* states without making *entry* metadata harder to read.

The *transform* names may also evolve.
The current names are compact and memorable,
but they should remain subordinate to the model they describe.
If the project learns a clearer vocabulary,
*entries* and manuals can internalize that vocabulary deliberately.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from): (none)

> **Sirno generated links end.**
