---
name: Misty Lake
desc: A projected, editable lake workspace rendered by a mist.
category:
  - concept
  - implemented
belongs:
  - sirno-lake
  - storage
  - generated-navigation
prerequisite:
  - reservoir
  - mist
refines:
  - sirno-lake
---

A *misty lake* is a materialized projection of reservoir entries.

A misty lake is produced by a *mist*.
It uses the same entry-address layout as the reservoir by default,
so existing lake reading habits still work.
A default project mist may render into `sirno-lake/`
while the canonical reservoir lives at `.sirno/lake`.
The reservoir remains the whole lake for metadata, structural checks, and generated navigation.
A misty lake may contain only the selected entries.

A misty lake is a working surface.
Humans, agents, editors, and local tools may read and edit it directly.
Those edits are *mist ripples* until explicit intake writes them back into the reservoir.
Anchor update should refuse to accept the reservoir while an editable misty lake has
unintaken ripples, stale state, conflicts, or staged workspace files.

A misty lake carries a local manifest at `.sirno/mist.toml` inside the projection.
The manifest identifies the mist spec,
records the source entry fingerprints, selector, projection settings,
and render settings used for rendering,
and lets intake detect staleness and conflicts without relying on timestamps.

All renders happen in misty lakes.
Generated footers, generated indexes, route files, or other Sirno-owned presentation output
belong in the projected workspace.
They are computed from the checked reservoir and written onto selected projected entries.
The reservoir remains the canonical authored store.
