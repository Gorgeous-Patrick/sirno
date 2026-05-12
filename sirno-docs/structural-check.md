---
name: Structural Check
description: Validation of entry shape, relation targets, footers, and witnesses.
category:
  - concept
clustee:
  - sirno-store
---

Sirno checks structure.

Structural checks include required metadata fields, accepted field shapes,
reference existence, generated footer boundaries,
and witness lookup validity when requested.

During editing, dangling `category`, `clustee`, and `refiner` ids may warn.
At an explicit review boundary, those dangling ids are errors.

Checks keep local movement fast while making review boundaries strict.
They do not decide whether prose is true or whether code satisfies a claim.

The check boundary matters because editing and review have different needs.
During editing, a person may create an entry before creating every related target.
Warnings keep that work visible without blocking movement.
At review time, dangling references should be fixed,
because the store is being treated as a coherent design surface.

File checks keep the public store shape predictable.
An entry directory contains Markdown entry files with valid ids.
Each file starts with accepted frontmatter.
Generated footer sentinels must be well formed.
Unknown fields and malformed relation values are errors because tools cannot safely infer intent from them.

Relation checks keep the graph navigable.
If an entry categorizes itself by an id,
clusters under an id,
or refines an id,
that target should exist by the time the store is reviewed.
This lets query results, generated footers, and reader navigation agree about the same set of entries.

Semantic review remains human and agent work.
The checker can say that a witness marker is shaped correctly.
It cannot say that the witnessed code is a good implementation of the claim.
That distinction keeps Sirno useful without pretending to solve judgment.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [sirno-store](sirno-store.md)
> **Sirno generated links end.**
