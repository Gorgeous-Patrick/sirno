---
desc: A project-owned editable lakelet.
name: Local Lakelet
category:
  - concept
belongs:
  - lake-composition
  - lake-namespace
prerequisite:
  - lakelet
  - anchor
---

A *local lakelet* is a project-managed lakelet.

It is implicit.
A project creates one by adding unmanaged entries under a domain folder.
The folder's entry domain becomes the local lakelet prefix.
No `Sirno.toml` table declares the local lakelet.

Local lakelets are project-owned editable content.
They do not carry the `managed` frozen reason by default.
An upstream declaration can claim the same domain path.
If `[upstreams.core]` exists,
then `lake/core/` is a glacier owned by crystallization.
Sirno rejects crystallization when unmanaged local files already occupy that domain.

Anchor records local lakelet entries by their flattened entry addresses.
There is no separate lakelet baseline.
