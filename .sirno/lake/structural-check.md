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

Structural checks cover these areas:

| Area | What it validates |
|---|---|
| metadata shape | Required fields and accepted field shapes. |
| structural targets | References through fields with structural relation entries. |
| relation entries | `meta.type: "structural"` entries use valid relation field names. |
| intrinsic entries | `meta.type: "intrinsic"` entries use valid intrinsic field names. |
| meta registry | Raw metadata scan finds intrinsic and structural field entries. |
| category targets | Entries used as category targets include `category: category`. |
| generated footers | Sirno-owned footer boundaries and freshness. |
| witnesses | Configured witness lookup validity when requested. |

When `category` metadata is present or the `category` relation entry exists,
checks warn if the `category` *entry* is missing.
When `[repo].members` is configured,
review checks report *repository witness* blocks that name missing *entries*.
They also report configured *witness* delimiters that are not part of a complete block.

Generated footer checking has two parts.
Sentinel shape is always checked.
Freshness is controlled by `[check].render`,
which is enabled by default.

## Check Command

`sirno check` checks the configured reservoir.
The `-m, --mode` option selects the check boundary.
Mist render derives generated-footer freshness from the checked reservoir and the selected projection.

Edit and review modes use different severity boundaries:

| Diagnostic | Edit mode | Review mode |
|---|---|---|
| dangling structural link target | warning | error |
| list-valued metadata without a structural relation entry | warning | warning |
| invalid structural relation field name | warning | error |
| invalid intrinsic field name | warning | error |

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
The first load phase scans raw frontmatter for `meta.type`.
The second phase parses entries with the generated meta registry.
Files may use LF or CRLF line endings.
Mixed LF and CRLF line endings warn,
because a file should keep one line-ending style even when Sirno can still parse it.
Generated footer sentinels must be well formed.
Malformed structural link values are errors because tools cannot safely infer target ids from them.
Uninhabited structural link relations are warnings
because the *entry* names structure the lake does not define.

Metadata target checks keep the graph navigable.
If an *entry* names a target through discovered structural link metadata,
that target should exist by the time the *lake* is reviewed.
This lets query results, generated footers, tide workitems,
and reader navigation agree about the same set of *entries*.

Intrinsic metadata field entries keep Sirno's required entry shape self-described.
The checker accepts `meta.type: "intrinsic"` on entries with valid intrinsic field names.
The raw scan registers those entries before typed parsing.
Their type marker binds them to fields the entry parser requires.

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
