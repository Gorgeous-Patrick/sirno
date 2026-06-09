---
desc: The Git-backed accepted baseline for Sirno Lake review.
name: Anchor
category:
  - implemented
  - concept
belongs:
  - lake-review
prerequisite:
  - lake
  - project-config
refines:
  - versioning
---

Anchor is the accepted-baseline subsystem for a Sirno Lake.

It records the reviewed Sirno Lake state as tracked semantic fingerprints.
Git owns history, branching, restore, retention, and destructive history operations.
Anchor owns the accepted design baseline
and the Tide comparison against that baseline.

Anchor is intentionally small.
It records enough accepted state for comparison,
then lets Git preserve every historical version of that state.
It does not store old entry bodies, private snapshots, checkout state, or retention policy.

Its local contract is accepted-state comparison:

- write the accepted baseline to `.sirno/anchor.toml`;
- compare the current waterline against that baseline;
- expose current ripples through project status;
- accept a new baseline only after review-mode checks and Tide review pass;
- clear obsolete Tide review state after the current waterline is accepted.

## Command Surface

`anchor-commands` owns Anchor command spelling and behavior.
This entry owns the accepted-baseline subsystem contract.

## Related Design Entries

Anchor is the subsystem boundary.
Smaller entries own the concrete file-state contracts.
These related entries are the review route through those contracts:

- *Anchor File* defines `.sirno/anchor.toml` and fingerprint semantics.
- *Control Files* defines `.sirno/` placement, target file ownership, and merge validity.
- *Tide* defines how Anchor differences become review work.
- *Tide Resolution* and *Tide Review File* define persisted review state.
- *Upstream File* defines upstream dependency pins.
- *Versioning* defines the boundary between Git history and Sirno accepted baselines.

## Accepted Baseline Behavior

Project status compares the current waterline against `.sirno/anchor.toml`.
If Anchor is absent, Tide treats the current lake as added against an empty baseline.
Anchor update writes a new `.sirno/anchor.toml`
after review-mode checks pass,
Tide has no open workitems,
and the default editable mist has no pending ripples or blockers.
The update also clears active Tide review state
because the current waterline has become the accepted baseline.

Anchor keeps history responsibilities in Git.
It stores accepted fingerprints for comparison,
not old entry bodies, private snapshots, checkout state, or retention policy.
Entry-owned artifacts stay part of the lake state through owner artifact-tree fingerprints.
