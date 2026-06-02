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
- Review receipt archival should define which accepted review records are kept,
  which records can be named,
  and how review interfaces expose them.
- Lake dependency management should refine *entry domain* resolution,
  symlink materialization,
  and upstream version selection without making entry names carry all dependency policy.
- The `extension-system` proposal defines executable entry artifacts
  that can run from Sirno hook points.
  Hook entries still need to define trigger points, payloads, result contracts, and failure policy.
- Future editing interfaces may provide a direct GUI or Obsidian-style experience.
  They should preserve the existing ownership rules:
  metadata is structural,
  generated footer regions are Sirno-owned,
  and prose outside generated regions remains user-owned.

Future work should remain explicit without becoming speculative architecture.
The current design is useful because its core is small:
*entries*, metadata, structural links, generated footers, forms, *transforms*, checks, and *witnesses*.
New features should preserve that clarity.

Git provides history and retention.
Sirno still needs long-term policy for optional review receipt archives.
That policy should preserve reviewable *lake* states without making *entry* metadata harder to read.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [extension-system](extension-system.md)

> **Sirno generated links end.**
