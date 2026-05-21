---
name: Entry Path
desc: A dot-separated lookup form that resolves to an entry.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry
  - entry-domain
---

An *entry path* is a dot-separated lookup form for finding an *entry*.

The syntax is `<id>.<id>` for one domain and one local name.
It may contain more segments, such as `<id>.<id>.<id>`,
when the lookup passes through nested *entry domains*.
The domain segments map to folders in the *lake*.
The final segment names the local entry position inside the last domain.

Paths that begin with `.<id>` are Sirno built-in paths.
They are reserved for tool-owned lake functionality,
such as the `.artifacts` space for *entry artifacts*.
Project entries and dependency domains use ordinary `<id>` segments instead.

An *entry path* is not necessarily a unique entry identity.
Several paths may resolve to the same *entry* through dependency links,
flattened views, or other resolution rules.
Use *entry id* for the stable identity an entry owns.
Use *entry path* for the address a reader or tool follows to find it.

Entry paths give Sirno a small syntax for composed lakes.
They keep the common flat lake easy to read,
while leaving room for nested domains and external dependency domains.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
