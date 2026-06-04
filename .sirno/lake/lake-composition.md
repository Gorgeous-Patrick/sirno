---
desc: The review neighborhood for composing local and upstream lakelets.
name: Lake Composition
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - sirno-lake
  - lake-namespace
---

Lake composition is the review neighborhood for building one addressable lake view from local and upstream parts.

It gathers the entries that decide how multiple lakelets become one lookup space:
`lake-namespace` owns entry-address domains and lookup rules,
`lake-system` names the set of lakelets used for lookup,
`lake-sheaf` is the resolved composition model,
and `sirno-upstream` owns Git-backed upstream declarations and crystallization.

The composition neighborhood is about the addressable view.
It keeps local entries, implicit local lakelets, upstream locks, and crystallized glaciers reviewable together
without making every upstream detail a direct neighbor of `sirno-lake`.

Review these entries together when a change affects domain lookup, lakelet discovery,
upstream crystallization, glacier protection, or composed entry resolution.
