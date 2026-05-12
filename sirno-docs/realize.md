---
name: Realize
description: The movement from Sirno entries into repository implementation.
category:
  - concept
clustee:
  - direction
refiner:
  - direction
---

`realize` moves from `sirno` to `code`.

Realizing uses entries to guide implementation.
Before editing code, read the entries that govern the work,
follow their category, clustee, refiner, and witness fields,
and inspect any witnessed repository regions.

A realization step should be able to answer which entry explains a local design commitment.
Not every line needs its own entry,
but important commitments need a nominal place.

Realization is where named design becomes behavior.
The entry store should tell the implementer what matters:
which concept is being made concrete,
which field or invariant must be preserved,
and which existing witnesses should be inspected before editing.

The code change should stay honest to the entry.
If the entry is still correct,
implementation can proceed under that name.
If implementation reveals that the entry is incomplete or misleading,
the work should include reflection so the store learns from the code.

This makes realization a two-way discipline.
The store guides code,
and code can expose pressure on the store.
The important part is that local implementation does not float free of design intent.
Future readers should be able to ask why a piece of code exists
and find the entry that gave the commitment a name.

---

> **Sirno generated links begin. Do not edit this section.**

Category (from): (none)

Category (to)
- [concept](concept.md)

Clique
- [direction](direction.md)
- [lower](lower.md)
- [raise](raise.md)
- [reflect](reflect.md)

> **Sirno generated links end.**
