---
name: Lake Sheaf
desc: A resolved view that glues multiple lakes into one addressable entry surface.
category:
  - concept
belongs:
  - lake-namespace
  - sirno-upstream
prerequisite:
  - lake-namespace
  - entry-address
---

A *lake sheaf* is the resolved view of a top-level *lake* and its lakelets.

The name describes the design model.
Local and crystallized lake data can be glued into one global view
when compatible *entry addresses* resolve to consistent entries.
The global view is what readers and tools navigate after resolution.

A lake sheaf resolves lakelets into one addressable entry surface before resolving dependency diamonds.
Dependency domains then link back to the resolved entries.
This keeps one composed entry surface even when several dependency paths reach the same lake.
Local lakelets and glaciers are lakelets.
Both use the same entry-address resolution
and are represented in anchor by their flattened entry addresses.

The sheaf model names composition, lookup, and consistency.
It does not choose dependency versions.
Version selection is separate future design.
