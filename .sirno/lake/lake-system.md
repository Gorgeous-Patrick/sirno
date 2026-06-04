---
name: Lake System
desc: One Sirno project together with its declared upstream lakes.
category:
  - concept
belongs:
  - lake-composition
  - sirno-upstream
prerequisite:
  - upstream-lake
---

A *lake system* is one Sirno project together with its declared upstream lakes.

The current project is the operating point.
Its upstream declarations describe the other Git-backed lakes that participate in the system.
Crystallization turns those upstream lakes into glaciers
in the local lake view that readers and tools can inspect.

A lake system is the operational group.
A *lake sheaf* is the resolved composition model for the addressable view.

| Term | Role |
|---|---|
| Lake system | The operational group: current project plus declared upstream lakes. |
| Lake sheaf | The resolved addressable view built from local and upstream lakelets. |
| Glacier | A crystallized upstream snapshot materialized under the local lake view. |
