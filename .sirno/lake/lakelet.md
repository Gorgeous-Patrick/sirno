---
name: Lakelet
desc: A folder-backed namespace surface for an entry domain.
category:
  - concept
belongs:
  - lake-composition
  - lake-namespace
prerequisite:
  - entry-domain
  - entry-address-resolution
---

A *lakelet* is the folder-backed namespace surface for an *entry domain*.

A lakelet gives entries under one domain prefix a shared storage and ownership boundary.
It may contain entries and nested lakelets.
The lakelet boundary is the namespace, not its depth in the lake directory.
`lake-namespace` owns the path examples for this model.

A lakelet can be project-managed or crystallization-managed.
A project-managed lakelet is a local lakelet.
A crystallization-managed lakelet formed from an upstream lake is a glacier.
Both use ordinary entry-address resolution through their domain.
