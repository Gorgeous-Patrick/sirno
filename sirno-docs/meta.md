---
name: Meta
description: A category for entries that define the project's Sirno-facing documentation method.
category:
  - meta
  - concept
---

`meta` classifies entries that define how a Sirno-managed project develops its documentation.

It is also the review front door for initialized vocabulary entries.
`concept` and `narrative` belong to it because changes to those entries
affect how the lake vocabulary is read.

The *entries* categorized by `meta` are ordinary entries;
notably, they are not privileged built-ins.
They give the *lake* a documented way to name categories, structural concepts,
reader perspectives, and local documentation habits.

The `meta` category is useful because Sirno keeps its own ontology small.
Instead of hard-coding a large list of kinds,
the project can create entries that define categories and classify those entries with `meta`.
This makes the vocabulary explicit and reviewable.

The central question for `meta` is:
how should this project's documentation develop?
The answer should be written as *entries*.
A project can define how it names concepts,
when it splits prose,
how it writes narratives,
which terms carry local meaning,
and how agents should update the *lake*.

This is the bootstrap claim:

> Sirno does not just provide documentation for the project;
> it lets the project document its own documentation method.
> Sirno does not just describe what a project is;
> it gives the project a place to describe how understanding should grow.

For example, `concept` and `narrative` can both be entries.
They explain how other entries are meant to be read,
but they still use the same metadata shape as every other entry.

That uniformity keeps the lake easy to reason about.
The same query, check, footer, and metadata rules apply to entries about vocabulary
and to entries about implementation commitments.

---

> **Sirno generated links begin. Do not edit this section.**

belongs (to): (none)

belongs (from):
- [concept](concept.md)
- [narrative](narrative.md)

> **Sirno generated links end.**
