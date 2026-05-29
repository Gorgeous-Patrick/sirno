---
name: Structural Link
desc: A metadata-backed relation that carries operational Sirno structure.
category:
  - concept
belongs:
  - sirno
  - meta-type
prerequisite:
  - metadata
---

A structural link is an entry-to-entry relation that Sirno reads as project structure.

This repository recommends the `category`, `belongs`, `prerequisite`, and `refines` relations.
`Sirno.toml` defines the active relation set with `[structural.FIELD]` subtables.
Each configured `FIELD` is the relation name.
The subtable's `entry` value names the *entry* that documents that relation.
Configured relations are ordinary *entry* metadata fields today,
but Sirno treats their values as the graph that powers query, checking,
generated footer rendering, and tide review worklists.

`meta.type: "structural"` marks an *entry* as the definition of one configured relation.
The matching relation entry documents that relation's meaning
and carries its Tide policy in `meta.ripple.lake` and `meta.ripple.anchor`.
The marker lets checks confirm that every configured relation has a documented owner.
It also keeps relation behavior local to the relation entry,
so rendering policy can live in config while review policy stays in the lake.
`Sirno.toml` defines relation order and generated-footer rendering.

Structural links refer to *entries* by path.
They are list-valued and may name several targets.
An empty list is still a present field.
Their configured order is user-managed.
Sirno uses that order when rendering configured structural surfaces.
Humans discover *witness* regions mechanically with `sirno witness ENTRY_ADDRESS --full`.
Agents use the corresponding MCP witness tool.

When `sirno entry rename OLD NEW` renames a relation entry,
it rewrites any `[structural.FIELD].entry = "OLD"` value to `NEW`.
It does not rename the metadata relation `FIELD`.
The same operation rewrites structural link target values that name `OLD`.

This *entry* is the review front door for the structural link relation *entries*.
It gives the relation set one review front door while leaving each relation *entry* free
to carry its own meaning and other `belongs` targets.

The *repository witness* for this *entry* should show the generic structural metadata map.
The active relation set and rendered directions are defined by `Sirno.toml`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [meta-type](meta-type.md)
  - [sirno](sirno.md)
- belongs (from):
  - [belongs](belongs.md)
  - [category](category.md)
  - [generated-navigation](generated-navigation.md)
  - [prerequisite](prerequisite.md)
  - [refines](refines.md)

> **Sirno generated links end.**
