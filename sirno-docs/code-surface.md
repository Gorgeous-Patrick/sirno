---
name: Code Surface
description: The repository artifacts that realize and witness design entries.
category:
  - concept
clustee:
  - sirno
refiner:
  - surface
---

The code surface is the repository implementation surface.

It includes source files, tests, configuration, generated files, assets,
and other artifacts that realize design decisions.

Repository artifacts can witness entries through `mosaika`.
Sirno uses the entry id as the witness query key,
keeping design names and repository marks connected without embedding mark syntax in entry prose.

The code surface is where design becomes costly in the useful sense.
Names, invariants, parser choices, storage boundaries, user interfaces,
tests, and generated assets all make commitments that future work must honor or revise.
Sirno does not ask every line of code to carry a design entry.
It asks important commitments to have a name that can survive beyond the edit that introduced them.

Witnesses make that name concrete.
An entry can state a claim,
and repository marks can show where the claim is implemented, tested, configured, or generated.
The mark belongs to the repository artifact.
The entry keeps the design language.
The shared key is the entry id.

This keeps code and documentation coupled without making either one awkward.
Code does not need long narrative comments for every design concept.
Entries do not need to duplicate source snippets that will drift.
Review can move between them by asking which entry explains a code commitment,
and which repository artifact witnesses an entry.

---

> **Sirno generated links begin. Do not edit this section.**

Category (from): (none)

Category (to)
- [concept](concept.md)

Clique
- [concept-driven-development](concept-driven-development.md)
- [future-work](future-work.md)
- [mono](mono.md)
- [planning](planning.md)
- [project-self-application](project-self-application.md)
- [sirno](sirno.md)
- [sirno-store](sirno-store.md)
- [storage-and-interfaces](storage-and-interfaces.md)
- [surface](surface.md)

> **Sirno generated links end.**
