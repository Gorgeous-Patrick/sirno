---
name: Reflect
description: The movement from changed code back into Sirno entries.
category:
  - direction
clustee:
  - direction
refiner:
  - direction
---

`reflect` moves from `code` to `sirno`.

Reflection records durable design facts learned during implementation.
Reflect when code changes a representation, narrows an invariant,
introduces a boundary, invalidates an explanation,
or reveals a clearer local design than the current entries record.

Reflection should happen while the code change is fresh.
The reflected entry records the design fact future work needs,
not a full narrative of the edit.

Good reflection asks what the repository now knows that the store should know too.
A function rename may not matter.
A new storage boundary, parser invariant, CLI contract, or test rationale usually does.
The reflected prose should name the stable design fact,
then connect it to existing entries through metadata when structure is useful.

Reflection should avoid turning the store into a changelog.
The commit history can explain the sequence of edits.
The entry store should explain the design that survived the edit.
If a change was exploratory and later discarded,
it may not deserve reflection.
If a change changes how future work should reason,
it should be reflected while the reason is still clear.

Reflection keeps the monograph and store from becoming ceremonial.
It gives implementation a way to improve the design model,
not merely comply with it.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [direction](direction.md)
> **Sirno generated links end.**
