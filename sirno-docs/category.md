---
name: Category
description: A structural field that classifies an entry by other entries.
category:
  - concept
belongs:
  - structural-field
---

`category` classifies an entry by other entries.

Categories are themselves entries.
This keeps the project vocabulary open and documented instead of fixed by Sirno.

Meta-classification uses the same mechanism.
The category id `meta` classifies entries that define categories,
including the initialized `concept` and `narrative` entries.
A category target should therefore be categorized by `meta`.
This makes each usable category a documented part of the project's Sirno method.

The reserved `locked` field may later protect entries or regions that a project treats as controlled.

Use `category` when the classified entry should be read as an instance of a named kind.
An entry categorized by `concept` should define a compressed idea.
An entry categorized by `narrative` should record or name a route through concepts.
An entry categorized by `meta` should define project vocabulary or documentation method.

Because categories are entries,
their meanings can be documented in the same lake they classify.
This avoids a hidden enum in the implementation becoming the only source of truth.
The project can grow vocabulary by adding entries,
and Sirno can still check that category targets are documented as categories.

Categories should stay semantic rather than decorative.
If a label only helps browsing by topic,
`belongs` may be a better fit.
If an entry makes another entry more concrete,
`refines` is the sharper field.
The category field is most useful when it tells the reader what kind of object they are looking at.

---

> **Sirno generated links begin. Do not edit this section.**

belongs (to):
- [structural-field](structural-field.md)

belongs (from): (none)

> **Sirno generated links end.**
