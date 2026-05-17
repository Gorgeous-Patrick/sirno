---
name: Structural Field
desc: A metadata field that carries operational Sirno structure.
category:
  - concept
belongs:
  - sirno
---

A structural field is a configured metadata field that Sirno reads as project structure.

This repository recommends `category`, `belongs`, and `refines`.
`Sirno.toml` defines the active set under `[structural]`.
Configured fields are ordinary *entry* metadata,
but Sirno treats their values as the graph that powers query, checking,
generated footer rendering, and tide review worklists.

Structural fields refer to *entries* by id.
They are list-valued and may name several targets.
Their configured order is user-managed.
Sirno uses that order when rendering configured structural surfaces.
Agents should discover *witness* regions mechanically with `sirno witness ENTRY_ID --full`.

This *entry* is the review front door for the structural field *entries*.
It gives the field set one review front door while leaving each field *entry* free
to carry its own meaning and other `belongs` targets.

The *repository witness* for this *entry* should show the generic structural metadata map.
The active field set is defined by `Sirno.toml`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [belongs](belongs.md)
  - [category](category.md)
  - [refines](refines.md)

> **Sirno generated links end.**
