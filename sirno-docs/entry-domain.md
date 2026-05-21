---
name: Entry Domain
desc: A named entry path segment that maps to a lake folder.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry
  - sirno-lake
---

An *entry domain* is a named segment in an *entry path*.

A domain segment maps to one folder inside a *lake*.
Nested domains form nested folders.
For example, `core.design` names the `core/` domain,
then the `design` local entry name inside that domain.
When written as a prefix, the domain includes the trailing dot:
`core.` in `core.design`.

Project-defined domains use ordinary `<id>` segments.
The leading-dot form `.<id>` is reserved for Sirno built-in functionality.
For example, `.artifacts` is the built-in lake space for *entry artifacts*.

An *entry domain* is an address space, not the identity of one *entry*.
A domain can contain entries and other domains.
It lets a project group entries by origin, subject, module, or dependency boundary
without making that grouping part of the entry's durable identity.

Use domain names as lowercase ASCII ids by default.
Keep them short and stable.
A domain should earn its name by reducing collisions,
clarifying ownership,
or making a composed lake easier to navigate.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
