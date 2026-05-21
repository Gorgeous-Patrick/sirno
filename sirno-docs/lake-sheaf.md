---
name: Lake Sheaf
desc: A resolved view that glues multiple lakes into one addressable entry surface.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry-path
  - sirno-lake
refines:
  - future-work
---

A *lake sheaf* is the resolved view of a top-level *lake* and its dependency lakes.

The name describes the design model.
Local lake data can be glued into one global view
when compatible *entry paths* resolve to consistent entries.
The global view is what readers and tools navigate after resolution.

A lake sheaf flattens sublakes to the top level before resolving dependency diamonds.
Dependency domains then link back to the top-level resolved entries.
This keeps one composed entry surface even when several dependency paths reach the same lake.

The sheaf model names composition, lookup, and consistency.
It does not choose dependency versions.
Version selection is separate future design.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
