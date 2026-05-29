---
name: Sirno Upstream
desc: The subsystem for Git-backed upstream lakes and crystallization.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - project-config
  - sirno-lock
  - lake-namespace
---

*Sirno Upstream* is the subsystem for declaring Git-backed upstream lakes,
locking them to exact Git commits,
and crystallizing them into glaciers in the current lake.

The subsystem gives one handle to the operational dependency model:
upstream declarations,
the lake system formed by those declarations,
and crystallization of glaciers.

Every upstream is included through crystallization.
The resulting glacier uses the glacier domain as its entry-address prefix,
and Sirno protects the glacier files with the `managed` frozen reason.
The glacier domain is an explicit local name in `Sirno.toml`.
It has no default derived from the Git source.
It shares its lake path with implicit local lakelets,
so an unmanaged local folder blocks crystallization for the same domain.

A lake sheaf remains the composition model for the resolved addressable view.
Sirno Upstream is the operator-facing feature that produces that local view.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from):
  - [crystallization](crystallization.md)
  - [glacier](glacier.md)
  - [lake-sheaf](lake-sheaf.md)
  - [lake-system](lake-system.md)
  - [upstream-lake](upstream-lake.md)

> **Sirno generated links end.**
