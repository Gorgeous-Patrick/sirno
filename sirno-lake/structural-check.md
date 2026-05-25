---
name: Structural Link Check
desc: Validation of entry shape, structural link targets, footers, and witnesses.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - metadata
  - generated-footer
  - sirno-witness
---

Sirno checks structure.

Structural checks include required metadata fields, accepted field shapes,
reference existence, generated footer boundaries,
and *witness* lookup validity when requested.
They check references through fields configured by `[structural.FIELD]` subtables.
Each configured link relation must also name an existing *entry*
with `meta.type: "structural"`.
That structural-inhabitance check is controlled by `[check].structural-inhabitance`
and is enabled when the flag or the whole `[check]` table is absent.
Entries named `name` or `desc` define Sirno's intrinsic metadata fields.
When those entries exist,
they must carry `meta.type: "intrinsic"`.
An entry with `meta.type: "intrinsic"` must be either `name` or `desc`.
Category metadata also has a semantic target check.
Every *entry* used as a `category` target must itself include `category: category`.
When `category` metadata is present or `[structural.category]` is configured,
checks warn if the `category` *entry* is missing.
When `[repo].members` is configured,
review checks report *repository witness* blocks that name missing *entries*.
They also report configured *witness* delimiters that are not part of a complete block.

Generated footer checking has two parts.
Sentinel shape is always checked.
Freshness is controlled by `[check].render`,
which is enabled by default.

During editing, dangling structural link targets may warn.
At an explicit review boundary, dangling references are errors.
List-valued metadata fields that are absent from `[structural]` always warn.
Configured link relations with missing *entries* or missing `meta.type: "structural"`
follow the same edit warning and review error boundary as dangling structural link targets.
An entry with `meta.type: "structural"` that is not configured in `Sirno.toml`
also follows that boundary.
Intrinsic-field `meta.type` diagnostics follow that boundary too.

Checks keep local movement fast while making review boundaries strict.
They do not decide whether prose is true or whether code satisfies a claim.

The check boundary matters because editing and review have different needs.
During editing, a person may create an *entry* before creating every related target.
Warnings keep that work visible without blocking movement.
At review time, dangling references should be fixed,
because the *lake* is being treated as a coherent design form.

File checks keep the Sirno Lake shape predictable.
An *entry* directory contains Markdown *entry* files with valid ids.
Each file starts with accepted frontmatter.
Files may use LF or CRLF line endings.
Mixed LF and CRLF line endings warn,
because a file should keep one line-ending style even when Sirno can still parse it.
Generated footer sentinels must be well formed.
Malformed structural link values are errors because tools cannot safely infer target ids from them.
Unconfigured structural link relations are warnings
because the *entry* names structure the project config does not enable.

Metadata target checks keep the graph navigable.
If an *entry* names a target through configured structural link metadata,
that target should exist by the time the *lake* is reviewed.
If `Sirno.toml` configures a link relation,
the relation name should also exist as the *entry* that documents that relation.
That relation entry should define `meta.type: "structural"`,
even when the relation has no tide behavior.
`[check].structural-inhabitance` controls that configured-relation entry check.
This lets query results, generated footers, tide workitems,
and reader navigation agree about the same set of *entries*.

Intrinsic metadata field entries keep Sirno's built-in entry shape self-described.
The checker accepts `meta.type: "intrinsic"` only for the `name` and `desc` entries.
Those entries are stored and versioned like ordinary entries,
but their type marker binds them to fields the entry parser always requires.

Category target checks keep kind vocabulary explicit.
An *entry* that appears in another *entry*'s `category` list is a category target.
That target must include `category: category`,
so a reader can tell which *entries* may classify other *entries*.
Missing markers follow the edit-warning and review-error boundary.
The missing `category` *entry* diagnostic is always a warning,
because it points to the default vocabulary entry rather than one authored edge.

Semantic review remains human and agent work.
The checker can say that a *witness* block is shaped correctly.
It can report configured *witness* blocks that refer to missing *entry addresses*.
It can report an opening or closing *witness* delimiter that has no matching delimiter.
It cannot say that the witnessed *repository* artifact is a good implementation of the claim.
That distinction keeps Sirno useful without pretending to solve judgment.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
