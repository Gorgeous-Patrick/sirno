---
name: Entry
description: A named Markdown document in the Sirno store.
category:
  - concept
clustee:
  - sirno-store
---

An entry is a Markdown file in the Sirno store.

The filename stem is the entry id.
The id is globally unique inside the store,
case-sensitive, and written as lowercase ASCII kebab-case with optional digits.

Each entry has a YAML metadata block and a prose body.
The required metadata fields are `name` and `description`.
The structural fields are `category`, `clustee`, `refiner`, and `witness:`.

An entry should be readable in about five minutes or less.
It can state a concept, category, clique closure, refinement, invariant,
interface, implementation commitment, witnessable claim, or narrative route.

The body of an entry should be useful prose, not just a label.
It should tell a future reader what the entry means,
why it deserves a stable name,
and how it participates in the project model.
When the entry describes a local implementation commitment,
the body should explain the durable design fact rather than narrating the most recent edit.

The metadata block carries structure that tools must read exactly.
The body carries judgment, examples, and explanation.
This split lets Sirno stay simple.
It can validate ids and structural fields without pretending to understand the full meaning of the prose.

Good entries are compact but not cryptic.
They avoid repeating the entire monograph,
but they also give enough context that a reader can follow a query result without opening ten files.
If a concept depends on several other concepts,
the structural fields should carry the navigational structure,
and the prose should explain the local meaning in ordinary language.

---

> **Sirno generated links begin. Do not edit this section.**

Category (from): (none)

Category (to)
- [concept](concept.md)

Clique
- [generated-footer](generated-footer.md)
- [metadata](metadata.md)
- [project-config](project-config.md)
- [query](query.md)
- [sirno-store](sirno-store.md)
- [structural-check](structural-check.md)

> **Sirno generated links end.**
