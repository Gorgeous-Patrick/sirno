---
name: Prerequisite
desc: A structural link relation that defines a knowledge dependency between entries.
category:
  - concept
belongs:
  - structural
  - meta-type-structural
prerequisite:
  - structural
meta.type: "structural"
meta.ripple.lake: ["to", "from"]
meta.ripple.frost: ["from"]
---

`prerequisite` records a knowledge dependency from an *entry*
to the *entries* a reader should understand first.

The target *entry* provides vocabulary, context, or a prior commitment
that unlocks the current *entry*.
The current *entry* points to its prerequisites because the dependency matters
for reading, reviewing, or acting on the claim.

Use `prerequisite` when an *entry* answers the question:
what should the reader already know before this makes sense?
The field is most useful for narrative routes, advanced concepts,
command contracts, generated artifacts, and implementation commitments
that rely on earlier project objects.

Choose the nearest useful prerequisite.
Do not list every ancestor or every familiar term.
A prerequisite should reduce future search cost or prevent a likely misunderstanding.
An *entry* may name several prerequisites when each one unlocks a distinct part
of the current claim.

`prerequisite` is a dependency edge, not a kind, review neighborhood, or refinement.
Use `category` for kind.
Use `belongs` for review locality.
Use `refines` when the current *entry* makes a broader design claim more concrete.

This entry's `meta.ripple.lake` and `meta.ripple.frost` lists use direct `to` and `from` edges
for *tide* review workitems.
It does not use clique expansion,
because a knowledge dependency is directional.
Waterline `to` catches the dependencies named by the edited *entry*.
Waterline and frostline `from` catch entries that currently or formerly depend on it.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [meta-type-structural](meta-type-structural.md)
  - [structural](structural.md)
- belongs (from): (none)

> **Sirno generated links end.**
