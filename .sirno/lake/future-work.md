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

Future Work records design areas that should stay visible
before they are stable enough for local entries.
A reservation names pressure and the next design boundary.
It is not a feature promise or a substitute for the entry that will later own the design.

A future-work item should stay small:

- name the area;
- describe the unresolved design question;
- point to a proposal entry when one exists;
- leave implementation detail to the entry that later owns it.

Reserved areas are:

- `locked` should define how *entries*, metadata fields, or generated regions resist accidental edits.
  It needs a clear ownership model before it becomes part of the schema.
- Review receipt archival should define which accepted review records are kept,
  which records can be named,
  and how review interfaces expose them.
- Lake dependency management should refine *entry domain* resolution,
  symlink materialization,
  and upstream version selection without making entry names carry all dependency policy.
- `extension-system` proposes charms that resolve into spells invoked from Sirno hook points.
  Hook entries still need trigger points, payloads, result contracts, and failure policy.
- Future editing interfaces may provide a direct GUI or Obsidian-style experience.
  They should preserve existing ownership rules:
  metadata is structural,
  generated footer regions are Sirno-owned,
  and prose outside generated regions remains user-owned.

This list keeps possible work explicit without turning it into speculative architecture.
When a reservation gains stable semantics,
create or revise the local entry that owns those semantics and link it here only when the route helps.

The current Sirno design remains small:
*entries*, metadata, structural links, generated footers, forms, *transforms*, checks, and *witnesses*.
New features should preserve that clarity.
