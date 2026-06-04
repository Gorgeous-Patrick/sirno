---
name: Sirno Upstream
desc: The subsystem for Git-backed upstream lakes and crystallization.
category:
  - concept
belongs:
  - lake-composition
prerequisite:
  - project-config
  - upstream-file
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

## Command Surface

`upstream-commands` owns upstream command spelling and behavior.
This entry owns the subsystem contract and dependency model.
