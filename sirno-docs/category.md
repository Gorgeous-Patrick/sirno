---
name: Category
description: A relation that classifies an entry by other entries.
category:
  - relation
clustee:
  - relation
refiner:
  - relation
---

`category` classifies an entry by other entries.

Categories are themselves entries.
This keeps the project vocabulary open and documented instead of fixed by Sirno.

Meta-classification uses the same mechanism.
The category id `meta` classifies entries that define categories,
including the initialized `concept` and `narrative` entries.

The reserved `locked` field may later protect entries or regions that a project treats as controlled.

Use `category` when the classified entry should be read as an instance of a named kind.
An entry categorized by `concept` should define a compressed idea.
An entry categorized by `relation` should explain a structural connection.
An entry categorized by `direction` should explain movement between surfaces.

Because categories are entries,
their meanings can be documented in the same store they classify.
This avoids a hidden enum in the implementation becoming the only source of truth.
The project can grow vocabulary by adding entries,
and Sirno can still check that relation targets exist.

Categories should stay semantic rather than decorative.
If a label only helps browsing by topic,
`clustee` may be a better fit.
If an entry makes another entry more concrete,
`refiner` is the sharper relation.
The category field is most useful when it tells the reader what kind of object they are looking at.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [relation](relation.md)
> **Sirno generated links end.**
