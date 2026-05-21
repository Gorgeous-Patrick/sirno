---
name: Local Sublake
desc: An implicit project-owned top-level lake folder used as an entry domain.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry-domain
  - entry-address-resolution
  - sirno-frost
---

A *local sublake* is an ordinary top-level folder inside the configured lake.

It is implicit.
A project creates one by adding entries under a folder,
for example `lake/core/design.md`.
That file resolves to entry address `core.design`,
and `core.` is the local sublake's entry-domain prefix.
No `Sirno.toml` table declares the local sublake.

Local sublakes are project-owned editable content.
They use the same entry-address resolution as upstream lakes,
but they do not carry the `managed` frozen reason by default.
An upstream declaration claims the same top-level domain namespace.
If `[upstreams.core]` exists,
then `lake/core/` is owned by crystallization.
Sirno rejects crystallization when unmanaged local files already occupy that domain.

Frost stores local sublake entries in the flattened entry-address form.
For example, `lake/core/design.md` is stored as `core.design` in the current frost snapshot.
There is no separate sublake frost store or sublake-local snapshot.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
