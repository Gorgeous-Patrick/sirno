---
name: Lakelet
desc: A lake namespace used as an entry-domain surface.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry-domain
  - entry-address-resolution
---

A *lakelet* is a namespace inside a lake.

A lakelet is a domain feature.
It gives a folder-backed entry-address prefix to a group of entries.
For example, `lake/core/design.md` places `design` inside the `core.` lakelet
and gives it entry address `core.design`.
Nested domains can also be lakelets.
For example, `lake/core/runtime/scheduler.md` creates the nested `core.runtime.` lakelet
and gives the entry address `core.runtime.scheduler`.
The lakelet boundary is the namespace,
not its depth in the lake directory.

A lakelet can be project-managed or crystallization-managed.
A project-managed lakelet is a local lakelet.
A crystallization-managed lakelet formed from an upstream lake is a glacier.
Both use ordinary entry-address resolution through their domain.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
