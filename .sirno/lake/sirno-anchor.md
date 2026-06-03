---
name: Sirno Anchor
desc: The Git-backed accepted baseline for Sirno Lake review.
category:
  - implemented
  - concept
belongs:
  - sirno-lake
  - interfaces
  - storage
prerequisite:
  - sirno-lake
  - project-config
refines:
  - versioning
---

Sirno Anchor is the accepted-baseline subsystem for a Sirno Lake.

It records the reviewed Sirno Lake state as tracked semantic fingerprints.
Git owns history, branching, restore, retention, and destructive history operations.
Sirno Anchor owns the accepted design baseline
and the Tide comparison against that baseline.

Sirno Anchor is intentionally small.
It records enough accepted state for comparison,
then lets Git preserve every historical version of that state.
It does not store old entry bodies, private snapshots, checkout state, or retention policy.

Its local contract is accepted-state comparison:

- write the accepted baseline to `.sirno/anchor.toml`;
- compare the current waterline against that baseline;
- expose current ripples for operators and agents;
- accept a new baseline only after review-mode checks and Tide review pass;
- clear obsolete Tide review state after the current waterline is accepted.

## Anchor Operations

The current CLI and MCP surfaces expose Anchor through these operations:

| Operation | Behavior |
|---|---|
| `sirno anchor status` | Shows current lake ripples against `.sirno/anchor.toml`. |
| `sirno anchor check` | Validates `.sirno/anchor.toml` and compares it with the lake. |
| `sirno anchor update` | Accepts the current lake as the new baseline. |

`sirno anchor update` runs review-mode lake checks,
derives Tide from Anchor and the current waterline,
requires every open workitem to be resolved,
writes a new Anchor file,
and clears obsolete Tide review state.

The first update initializes Anchor from the current lake.
Later updates require a clear Tide.

## Related Design Entries

Sirno Anchor is the subsystem boundary.
The detailed storage contracts live in smaller entries.
These related entries are the review route through those contracts:

- *Anchor File* defines `.sirno/anchor.toml` and fingerprint semantics.
- *Sirno Control Files* defines `.sirno/` placement, target file ownership, and merge validity.
- *Sirno Tide* defines how Anchor differences become review work.
- *Tide Resolution* and *Tide Review File* define persisted review state.
- *Upstream File* defines upstream dependency pins.
- *Versioning* defines the boundary between Git history and Sirno accepted baselines.

## Current Implementation Notes

The first Anchor implementation provides `.sirno/anchor.toml`,
entry and artifact-tree fingerprints,
`sirno anchor status`,
`sirno anchor check`,
and `sirno anchor update`.
Tide compares the waterline against Anchor when the Anchor file exists.
If Anchor is absent, Tide treats the current lake as added against an empty baseline.

Temporary implementation surfaces remain while the target control-file split is actualized:

- structural relation entries spell the baseline-side policy as `meta.ripple.anchor`;
- merge drivers for `.sirno/anchor.toml`, `.sirno/tide.toml`, and `.sirno/upstream.toml`
  are not installed yet.

These surfaces are implementation debt, not new design direction.
The target design remains tracked `.sirno/anchor.toml`,
tracked active `.sirno/tide.toml`,
and tracked upstream dependency `.sirno/upstream.toml`.

## Excluded Snapshot Responsibilities

Sirno Anchor keeps snapshot responsibilities out of Sirno:

- private snapshot storage;
- snapshot commits;
- snapshot checkouts;
- snapshot garbage collection;
- Anchor-backed entry freeze checks;
- snapshot coordinates in lock state.

Entry-owned artifacts stay part of the lake state through owner artifact-tree fingerprints.
Upstream glaciers may still use managed local protection.
The manual `reviewed` freeze reason belongs to the deprecated entry-freeze design,
not to Anchor.
