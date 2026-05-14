---
name: Meta
description: A category for entries that define project vocabulary.
category:
  - meta
  - concept
---

`meta` classifies entries that define project vocabulary.

It is also the review front door for initialized vocabulary entries.
`concept` and `narrative` belong to it because changes to those entries
affect how the lake vocabulary is read.

Entries categorized by `meta` are ordinary entries.
They are not privileged built-ins.
They give the lake a documented way to name categories and structural concepts.

The `meta` category is useful because Sirno keeps its own ontology small.
Instead of hard-coding a large list of kinds,
the project can create entries that define categories and classify those entries with `meta`.
This makes the vocabulary explicit and reviewable.

For example, `concept` and `narrative` can both be entries.
They explain how other entries are meant to be read,
but they still use the same metadata shape as every other entry.

That uniformity keeps the lake easy to reason about.
The same query, check, footer, and metadata rules apply to entries about vocabulary
and to entries about implementation commitments.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from):
- [concept](concept.md)
- [narrative](narrative.md)

Belongs (to): (none)

> **Sirno generated links end.**
