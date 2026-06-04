---
desc: The review neighborhood for selecting and rendering lake views.
name: Lake Projection
category:
  - concept
belongs:
  - lake
prerequisite:
  - lake
  - reservoir
---

Lake projection is the review neighborhood for turning reservoir entries into readable workspaces.

It gathers the selector and projection concepts that make a lake visible outside its canonical store:
`query` selects entries,
`mist` stores a reusable selector and render settings,
`misty-lake` is the projected workspace,
and `generated-navigation` owns the Sirno-generated navigation surface.

The projection neighborhood is about views.
It does not own canonical entry content.
The reservoir remains the authored lake store.
Mist render and intake are the operations that move between the authored store and one projected view.

Review these entries together when a change affects selection, rendered layout, editable projections,
generated navigation, or intake behavior.
