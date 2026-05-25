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

Reserved areas are concrete:

- `locked` may define how *entries*, metadata fields, or generated regions resist accidental edits.
  That design needs a clear ownership model before it becomes part of the schema.
- Version retention should define which `eter` snapshots Sirno keeps by default,
  which snapshots can be named,
  and how review interfaces expose them.
- Lake dependency management should refine *entry domain* resolution,
  symlink materialization,
  and upstream version selection without making entry names carry all dependency policy.
- Future editing interfaces may provide a direct GUI or Obsidian-style experience.
  They should preserve the existing ownership rules:
  metadata is structural,
  generated footer regions are Sirno-owned,
  and prose outside generated regions remains user-owned.

Future work should remain explicit without becoming speculative architecture.
The current design is useful because its core is small:
*entries*, metadata, structural links, generated footers, forms, *transforms*, checks, and *witnesses*.
New features should preserve that clarity.

`eter` provides history, snapshots, retirement, and garbage collection.
`sirno frost gc` provides a latest-snapshot collection policy for frost rows and artifact bytes.
Sirno still needs long-term policy for which historical snapshots stay live.
That policy should preserve reviewable *lake* states without making *entry* metadata harder to read.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from): (none)

> **Sirno generated links end.**
