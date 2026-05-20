---
name: Category
desc: A structural field that classifies an entry by other entries.
category:
  - category
  - meta
  - concept
belongs:
  - structural-field
prerequisite:
  - structural-field
---

`category` classifies an *entry* by other *entries*.

Categories are themselves *entries*.
This keeps the project vocabulary open and documented instead of fixed by Sirno.

A category target must be usable as a kind,
and being usable as a kind is itself a documented property.
The `category` *entry* classifies the *entries* that may be used as category targets.
A category target should therefore be categorized by `category`.
This includes `category` itself and the initialized `concept`, `narrative`, and `meta` *entries*.
The marker is self-applied, which keeps the category vocabulary closed under its own rule.
`sirno check` reports a category target that lacks this marker.
It also warns when category metadata needs the `category` *entry* and that entry is missing.

The reserved `locked` field may later protect *entries* or regions that a project treats as controlled.

Use `category` when the classified *entry* should be read as an instance of a named kind.
An *entry* categorized by `meta` should define the project's principles, vocabulary, or documentation method.
An *entry* categorized by `category` may itself be used as a category target.
An *entry* categorized by `concept` should define a compressed idea.
An *entry* categorized by `narrative` should record or name a route through concepts.

Because categories are *entries*,
their meanings can be documented in the same *lake* they classify.
This avoids a hidden enum in the implementation becoming the only source of truth.
The project can grow vocabulary by adding *entries*.

Categories should stay semantic rather than decorative.
If a label only helps browsing by topic,
`belongs` may be a better fit.
If a label names earlier knowledge needed for understanding,
`prerequisite` is the sharper field.
If an *entry* makes another *entry* more concrete,
`refines` is the sharper field.
The category field is most useful when it tells the reader what kind of object they are looking at.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [structural-field](structural-field.md)
- belongs (from):
  - [concept](concept.md)
  - [meta](meta.md)
  - [narrative](narrative.md)

> **Sirno generated links end.**
