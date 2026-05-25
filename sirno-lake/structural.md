---
name: Structural Link
desc: A metadata-backed relation that carries operational Sirno structure.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - metadata
---

A structural link is an entry-to-entry relation that Sirno reads as project structure.

This repository recommends the `category`, `belongs`, `prerequisite`, and `refines` relations.
`Sirno.toml` defines the active relation set with `[structural.FIELD]` subtables.
Each configured `FIELD` is the relation name.
It should also name the *entry* that documents that relation
and follow normal *entry* id rules.
Configured relations are ordinary *entry* metadata fields today,
but Sirno treats their values as the graph that powers query, checking,
generated footer rendering, and tide review worklists.
The relation entry defines Tide behavior after declaring `meta.type: "structural"`.
`Sirno.toml` defines relation order and generated-footer rendering.

Structural links refer to *entries* by path.
They are list-valued and may name several targets.
An empty list is still a present field.
Their configured order is user-managed.
Sirno uses that order when rendering configured structural surfaces.
Humans discover *witness* regions mechanically with `sirno witness ENTRY_ADDRESS --full`.
Agents use the corresponding MCP witness tool.

Because a configured relation name is also a local *entry* id,
`sirno entry rename OLD NEW` treats that relation name as part of the rename.
It rewrites authored metadata keys from `OLD` to `NEW` across the *lake*
and rewrites `[structural.OLD]` in `Sirno.toml` to `[structural.NEW]`.
The same operation also rewrites structural link target values that name `OLD`.

This *entry* is the review front door for the structural link relation *entries*.
It gives the relation set one review front door while leaving each relation *entry* free
to carry its own meaning and other `belongs` targets.

The *repository witness* for this *entry* should show the generic structural metadata map.
The active relation set and rendered directions are defined by `Sirno.toml`.

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
