---
name: Mist
desc: A query-backed projector that selects lake entries and renders a misty lake.
category:
  - concept
  - implemented
belongs:
  - misty-lake
prerequisite:
  - query
  - reservoir
refines:
  - query
---

A *mist* is a selector and projector for a Sirno reservoir.

A mist chooses entries from the reservoir through the same selection model used by query.
Text terms, structural filters, field-state filters, and later graph expansion
should live in one shared selector mechanism.
`sirno query` prints the selected entries.
A mist renders the selected entries into a *misty lake*.

The mist is the filter itself,
not the projected directory.
It names what to show,
how to lay entry addresses out,
and whether the resulting workspace is editable.
It also owns projection settings,
including which structural relation edge directions render as generated navigation.
A project can keep shared mist specs under `.sirno/mist/`.
A user can keep local mist specs for personal or agent-specific workspaces.
The default shared mist spec is `.sirno/mist/default.toml`.

The implemented shared mist spec contains:

- `[projection].path` for the misty lake path, defaulting to `sirno-lake`;
- `[projection].editable` for intake eligibility, defaulting to `true`;
- `[select]` terms, exact terms, structural target filters, and structural state filters;
- `[render.structural]` generated navigation directions.

Edits made in a misty lake are *mist ripples*.
They are reviewable differences, not decay.
The term *drift* is reserved for unwanted or degraded divergence.

A mist should render entries with normal entry-address layout by default.
For example,
entry address `core.design` renders as `core/design.md` inside the misty lake.
That shape preserves the old lake browsing habit while keeping canonical storage in the reservoir.

A mist may also render Sirno-owned generated navigation.
Structural rendering belongs to the mist because it is presentation for one projection,
not canonical lake semantics.
Relation entries still own structural meaning and Tide review policy.
The reservoir supplies the metadata registry,
structural relation set,
and lake-wide link index used by the projection.
The misty lake does not need to contain the entries that define those lake semantics.
All rendered output belongs in misty lakes,
so the reservoir remains the authored source for entry metadata, prose, and artifacts.

Mist rendering projects selected reservoir entries into the misty lake,
copies selected entry artifacts,
renders generated navigation from the reservoir context,
and writes the projection manifest.
Mist status compares the projection with the reservoir.
Mist intake writes changed Markdown entries back to the reservoir
when the manifest is fresh and the projection is editable.

## Command Surface

`mist-commands` owns mist command spelling and behavior.
This entry owns selection, projection settings, and render semantics.
