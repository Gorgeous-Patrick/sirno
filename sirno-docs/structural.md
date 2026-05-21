---
name: Structural Field
desc: A metadata field that carries operational Sirno structure.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - metadata
---

A structural field is a configured metadata field that Sirno reads as project structure.

This repository recommends `category`, `belongs`, `prerequisite`, and `refines`.
`Sirno.toml` defines the active set with `[structural.FIELD]` subtables.
Each configured `FIELD` should also name the *entry* that documents that field
and follow normal *entry* id rules.
Configured fields are ordinary *entry* metadata,
but Sirno treats their values as the graph that powers query, checking,
generated footer rendering, and tide review worklists.

Structural fields refer to *entries* by path.
They are list-valued and may name several targets.
An empty list is still a present field.
Their configured order is user-managed.
Sirno uses that order when rendering configured structural surfaces.
Humans discover *witness* regions mechanically with `sirno witness ENTRY_ADDRESS --full`.
Agents use the corresponding MCP witness tool.

Because a configured field name is also a local *entry* id,
`sirno entry rename OLD NEW` treats that field name as part of the rename.
It rewrites authored metadata keys from `OLD` to `NEW` across the *lake*
and rewrites `[structural.OLD]` in `Sirno.toml` to `[structural.NEW]`.
The same operation also rewrites structural target values that name `OLD`.

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
  - [generated-navigation](generated-navigation.md)
  - [prerequisite](prerequisite.md)
  - [refines](refines.md)

> **Sirno generated links end.**
