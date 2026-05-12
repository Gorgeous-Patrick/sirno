---
name: Category
description: A structural field that classifies an entry by other entries.
category:
  - concept
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
An entry categorized by `narrative` should record or name a route through concepts.
An entry categorized by `meta` should define project vocabulary.

Because categories are entries,
their meanings can be documented in the same store they classify.
This avoids a hidden enum in the implementation becoming the only source of truth.
The project can grow vocabulary by adding entries,
and Sirno can still check that referenced entries exist.

Categories should stay semantic rather than decorative.
If a label only helps browsing by topic,
`clustee` may be a better fit.
If an entry makes another entry more concrete,
`refiner` is the sharper field.
The category field is most useful when it tells the reader what kind of object they are looking at.

---

> **Sirno generated links begin. Do not edit this section.**

- none

> **Sirno generated links end.**
