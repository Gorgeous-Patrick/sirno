---
name: Entry Address
desc: A dot-separated lookup form that resolves to an entry.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry
  - entry-atom
  - entry-domain
---

An *entry address* is a dot-separated lookup form for finding an *entry*.

The syntax is `<atom>.<atom>` for one domain and one local atom.
It may contain more atoms, such as `<atom>.<atom>.<atom>`,
when the lookup passes through nested *entry domains*.
The domain segments map to folders in the *lake*.
The final atom names the local entry position inside the last domain.

Addresses that begin with `.<atom>` are Sirno built-in addresses.
They are reserved for tool-owned lake functionality,
such as the `.artifacts` space for *entry artifacts*.
Project entries and dependency domains use ordinary `<atom>` segments instead.

An *entry address* is not necessarily a unique entry identity.
Several addresses may resolve to the same *entry* through dependency links,
flattened views, or other resolution rules.
Use *entry address* for the name a reader or tool follows to find an entry.

Entry addresses give Sirno a small syntax for composed lakes.
They keep the common flat lake easy to read,
while leaving room for nested domains and external dependency domains.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
