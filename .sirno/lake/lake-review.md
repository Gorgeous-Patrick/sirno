---
desc: The review neighborhood for lake checks, baselines, and review obligations.
name: Lake Review
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - sirno-lake
  - structural-check
  - sirno-anchor
---

Lake review is the review neighborhood for treating a reservoir waterline as coherent.

It gathers the entries that decide whether the lake is ready to accept as a baseline:
`structural-check` validates entry shape and structural references,
`sirno-anchor` records the accepted baseline,
`sirno-tide` reviews ripples against that baseline,
and `versioning` explains how Git and Anchor move together.

The review neighborhood owns the route through checks and baselines.
It does not replace the local contracts of Anchor, Tide, or structural validation.
Each leaf entry still owns its own command behavior, file shape, and repository witnesses.

Review these entries together when a change affects lake acceptance, generated-footer freshness,
witness checking, Tide workitems, or baseline update discipline.
