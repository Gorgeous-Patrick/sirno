---
name: Lake Namespace
desc: The domain and lakelet model for namespaced Sirno Lake entries.
category:
  - concept
belongs:
  - sirno-lake
  - lake-composition
prerequisite:
  - sirno-lake
  - entry-address-resolution
---

A *lake namespace* is the folder-backed address space inside a Sirno Lake.

The namespace model separates address syntax from storage ownership.
An *entry domain* is the address prefix.
A *lakelet* is the lake folder that realizes that prefix.
A *local lakelet* is project-owned.
A *glacier* is owned by crystallization.
A *lake sheaf* is the resolved addressable view after lakelets are composed.

| Term | Role |
|---|---|
| Entry domain | A non-final entry-address atom used as a namespace prefix. |
| Lakelet | The folder-backed namespace surface for an entry domain. |
| Local lakelet | A project-owned editable lakelet. |
| Glacier | A crystallization-owned managed lakelet. |
| Lake sheaf | The resolved entry surface formed from local lakelets and glaciers. |

The domain is the name.
The lakelet is the storage surface.
For example,
`lake/core/design.md` creates the `core.` domain,
uses `lake/core/` as the lakelet,
and resolves the entry as `core.design`.

Namespaces can nest.
For example,
`lake/core/runtime/scheduler.md` creates the `core.runtime.` domain
and resolves the entry as `core.runtime.scheduler`.
The namespace boundary is the domain prefix,
not its depth in the lake directory.

Ownership is exclusive at a domain path.
Unmanaged project files make a local lakelet.
An upstream declaration can claim the same domain as a glacier,
but crystallization rejects that claim while unmanaged files occupy the path.

Sirno Anchor records entries by flattened entry address.
It does not store separate namespace or lakelet baselines.
Git records the folder history.
