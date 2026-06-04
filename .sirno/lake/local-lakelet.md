---
name: Local Lakelet
desc: A project-owned editable lakelet.
category:
  - concept
belongs:
  - lake-composition
  - lake-namespace
prerequisite:
  - lakelet
  - sirno-anchor
---

A *local lakelet* is a project-managed lakelet.

It is implicit.
A project creates one by adding entries under a folder,
for example `lake/core/design.md`.
That file resolves to entry address `core.design`,
and `core.` is the local lakelet's entry-domain prefix.
No `Sirno.toml` table declares the local lakelet.

Local lakelets are project-owned editable content.
They do not carry the `managed` frozen reason by default.
An upstream declaration can claim the same domain path.
If `[upstreams.core]` exists,
then `lake/core/` is a glacier owned by crystallization.
Sirno rejects crystallization when unmanaged local files already occupy that domain.

Anchor records local lakelet entries by their flattened entry addresses.
For example, `lake/core/design.md` is recorded as `core.design`.
There is no separate lakelet baseline.
