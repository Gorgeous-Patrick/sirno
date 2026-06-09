---
desc: A reservoir view rendered through a mist.
name: Misty Lake
category:
  - concept
  - implemented
belongs:
  - lake
prerequisite:
  - lake
  - reservoir
  - mist
refines:
  - lake
---

A *misty lake* is a reservoir viewed through a mist.

A misty lake is the established form for working with a reservoir without making
the reservoir itself carry local presentation state.
The reservoir is quiet canonical storage.
People, agents, editors, and local tools need a readable working surface.
The mist supplies the selector, projection settings, and render settings.
The misty lake is the resulting view:
selected entries,
generated navigation,
editable workspace state,
and intake back into the reservoir.

The concept includes the projection pieces that make a lake visible outside its canonical store:
`query` selects entries,
`mist` stores a reusable selector and render settings,
and `generated-navigation` owns the Sirno-generated navigation surface.
Changes to selection, rendered layout, editable projections, generated navigation,
or intake behavior should read those entries together.

The misty lake is the view.
It does not own canonical entry content.
The reservoir remains the authored lake store.
Mist render and intake are the operations that move between the authored store
and one projected view.

A misty lake uses the same entry-address layout as the reservoir by default,
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

The *repository witnesses* for this entry should show projection settings,
the local manifest shape,
rendering from the reservoir into the projection,
and intake from the projection back into the reservoir.
