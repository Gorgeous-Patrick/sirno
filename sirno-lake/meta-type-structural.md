---
name: Structural Metadata Type
desc: The meta.type value for entries that define configured structural link relations.
category:
  - concept
  - meta
belongs:
  - meta-type
prerequisite:
  - meta-type
  - structural
---

`meta.type: "structural"` is the `meta.type` value for configured structural link relations.
It marks an *entry* as the definition of one configured relation.

`Sirno.toml` registers the relation name under `[structural.FIELD]`.
The matching *entry* documents that relation's meaning
and carries the relation's Tide policy in `meta.ripple.lake` and `meta.ripple.frost`.

The marker lets checks confirm that every configured relation has a documented owner.
It also keeps relation behavior local to the relation entry,
so rendering policy can live in config while review policy stays in the lake.

The active structural relation entries in this repository are `category`,
`belongs`,
`prerequisite`,
and `refines`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [meta-type](meta-type.md)
- belongs (from):
  - [belongs](belongs.md)
  - [category](category.md)
  - [prerequisite](prerequisite.md)
  - [refines](refines.md)

> **Sirno generated links end.**
