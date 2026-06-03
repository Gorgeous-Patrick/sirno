---
name: Lakelet
desc: A folder-backed namespace surface for an entry domain.
category:
  - concept
belongs:
  - lake-namespace
prerequisite:
  - entry-domain
  - entry-address-resolution
---

A *lakelet* is the folder-backed namespace surface for an *entry domain*.

A lakelet gives a group of entries a shared entry-address prefix.
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
