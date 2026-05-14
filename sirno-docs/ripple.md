---
name: Ripple
description: The named delta between two Sirno Lake states.
category:
  - concept
belongs:
  - sirno-lake
refines:
  - versioning
---

A ripple is the named delta between two Sirno Lake states.
It describes what changed across entries, metadata, generated projections,
or future frozen snapshots without treating the whole lake as new.

The term fits the lake model.
A lake is the readable body of project knowledge.
A ripple is a visible disturbance in that body:
small enough to inspect locally,
but meaningful because it belongs to a larger surface.

Ripple should name reviewable difference, not semantic judgment.
It can support future commands, displays, or persisted review artifacts that compare lake states.
Those interfaces may decide whether a ripple is transient output,
a durable record,
or a patch-like object,
but the concept remains the same delta between two states.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to):
- [sirno-lake](sirno-lake.md)

> **Sirno generated links end.**
