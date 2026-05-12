---
name: Project Self-Application
description: The way this repository uses its own Sirno model.
category:
  - concept
clustee:
  - sirno
---

This repository uses Sirno's own model.

`DESIGN.md` is the monograph.
The `sirno-docs/` directory is the Sirno store of compact entries for concepts,
structural fields, interfaces, and implementation commitments described by the monograph.

The codebase can witness entries through `mosaika`.

The monograph may grow long,
but it should remain ordered as one narrative.
Local details that become dense should be lowered into entries,
and raised back only when the monograph needs them.

This self-application is useful because it exercises the design under its own constraints.
The project can use `DESIGN.md` to explain Sirno as one continuous document,
then use entries to make the same design addressable in smaller pieces.
When implementation work changes the model,
the repository can reflect that change into the store before the monograph is raised again.

The store should not merely mirror the design document heading for heading.
It should name the objects the project expects future work to cite:
surfaces, entries, structural fields, directions, metadata,
checks, generated footers, witnesses, and storage boundaries.
Those names become the handles used by code work, documentation work, and review.

The codebase will make this stronger as witnesses are added.
When a Rust module realizes entry parsing,
generated footer handling,
or structural checks,
that code can be marked through `mosaika` under the relevant entry id.
Then the repository can answer both sides of the design question:
what does this entry mean,
and where does the code witness it?

---

> **Sirno generated links begin. Do not edit this section.**

Category (from): (none)

Category (to)
- [concept](concept.md)

Clique
- [code-surface](code-surface.md)
- [concept-driven-development](concept-driven-development.md)
- [future-work](future-work.md)
- [mono](mono.md)
- [planning](planning.md)
- [sirno](sirno.md)
- [sirno-store](sirno-store.md)
- [storage-and-interfaces](storage-and-interfaces.md)
- [surface](surface.md)

> **Sirno generated links end.**
