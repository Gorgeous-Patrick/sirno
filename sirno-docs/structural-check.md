---
name: Structural Check
description: Validation of entry shape, structural targets, footers, and witnesses.
category:
  - concept
belongs:
  - sirno-lake
---

Sirno checks structure.

Structural checks include required metadata fields, accepted field shapes,
reference existence, generated footer boundaries,
and witness lookup validity when requested.
They check references through fields configured under `[structural]`.
They also check that every configured `category` target is itself categorized by `meta`.
When `[repo].members` is configured,
review checks report repository witness blocks that name missing entries.

Generated-link checking has two parts.
Sentinel shape is always checked.
Freshness is controlled by `[check].link`,
which is enabled by default.

During editing, dangling structural ids may warn.
Category targets that are not categorized by `meta` also warn during editing.
At an explicit review boundary, those diagnostics are errors.
List-valued metadata fields that are absent from `[structural]` always warn.

Checks keep local movement fast while making review boundaries strict.
They do not decide whether prose is true or whether code satisfies a claim.

The check boundary matters because editing and review have different needs.
During editing, a person may create an entry before creating every related target.
Warnings keep that work visible without blocking movement.
At review time, dangling references should be fixed,
because the lake is being treated as a coherent design form.

File checks keep the public lake shape predictable.
An entry directory contains Markdown entry files with valid ids.
Each file starts with accepted frontmatter.
Generated footer sentinels must be well formed.
Malformed structural values are errors because tools cannot safely infer target ids from them.
Unconfigured structural fields are warnings because the entry names structure the project config does not enable.

Metadata target checks keep the graph navigable.
If an entry categorizes itself by an id,
belongs under an id,
or refines an id,
that target should exist by the time the lake is reviewed.
If an entry uses an id as a category,
that target should define project vocabulary or documentation method through `meta`.
This lets query results, generated footers, and reader navigation agree about the same set of entries.

Semantic review remains human and agent work.
The checker can say that a witness block is shaped correctly.
It can report configured witness blocks that refer to missing entry ids.
It cannot say that the witnessed repository artifact is a good implementation of the claim.
That distinction keeps Sirno useful without pretending to solve judgment.

---

> **Sirno generated links begin. Do not edit this section.**

belongs (to):
- [sirno-lake](sirno-lake.md)

belongs (from): (none)

> **Sirno generated links end.**
