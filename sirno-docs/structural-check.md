---
name: Structural Check
desc: Validation of entry shape, structural targets, footers, and witnesses.
category:
  - concept
belongs:
  - sirno-lake
---

Sirno checks structure.

Structural checks include required metadata fields, accepted field shapes,
reference existence, generated footer boundaries,
and *witness* lookup validity when requested.
They check references through fields configured by `[structural.FIELD]` subtables.
When `[repo].members` is configured,
review checks report *repository witness* blocks that name missing *entries*.

Generated footer checking has two parts.
Sentinel shape is always checked.
Freshness is controlled by `[check].render`,
which is enabled by default.

During editing, dangling structural ids may warn.
At an explicit review boundary, dangling references are errors.
List-valued metadata fields that are absent from `[structural]` always warn.

Checks keep local movement fast while making review boundaries strict.
They do not decide whether prose is true or whether code satisfies a claim.

The check boundary matters because editing and review have different needs.
During editing, a person may create an *entry* before creating every related target.
Warnings keep that work visible without blocking movement.
At review time, dangling references should be fixed,
because the *lake* is being treated as a coherent design form.

File checks keep the public *lake* shape predictable.
An *entry* directory contains Markdown *entry* files with valid ids.
Each file starts with accepted frontmatter.
Files may use LF or CRLF line endings.
Mixed LF and CRLF line endings warn,
because a file should keep one line-ending style even when Sirno can still parse it.
Generated footer sentinels must be well formed.
Malformed structural values are errors because tools cannot safely infer target ids from them.
Unconfigured structural fields are warnings because the *entry* names structure the project config does not enable.

Metadata target checks keep the graph navigable.
If an *entry* names a target through configured structural metadata,
that target should exist by the time the *lake* is reviewed.
This lets query results, generated footers, tide workitems,
and reader navigation agree about the same set of *entries*.

Semantic review remains human and agent work.
The checker can say that a *witness* block is shaped correctly.
It can report configured *witness* blocks that refer to missing *entry* ids.
It cannot say that the witnessed *repository* artifact is a good implementation of the claim.
That distinction keeps Sirno useful without pretending to solve judgment.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
