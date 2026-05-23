---
name: Local Lakelet
desc: An implicit project-owned lake folder used as an entry domain.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - lakelet
  - sirno-frost
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

Frost stores local lakelet entries in the flattened entry-address form.
For example, `lake/core/design.md` is stored as `core.design` in the current frost snapshot.
There is no separate lakelet frost store or lakelet-local snapshot.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
