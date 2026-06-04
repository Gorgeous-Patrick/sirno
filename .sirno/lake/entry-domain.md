---
desc: A non-final entry-address atom used as a namespace prefix.
name: Entry Domain
category:
  - concept
belongs:
  - entry-address-resolution
  - lake-namespace
prerequisite:
  - entry-atom
  - lake
---

An *entry domain* is a non-final *entry atom* used as an address prefix.

A domain atom maps to one folder inside a *lake*.
Nested domains form nested folders.
For example, `core.design` names the `core/` domain,
then the `design` local entry atom inside that domain.
When written as a prefix, the domain includes the trailing dot:
`core.` in `core.design`.

Project-defined domains use ordinary entry atoms.
The leading-dot form `.<atom>` is reserved for Sirno built-in functionality.
For example, `.artifacts` is the built-in lake space for *entry artifacts*.

An *entry domain* is an address space, not the identity of one *entry*.
A domain can contain entries and other domains.
It lets a project group entries by origin, subject, module, or dependency boundary
without making that grouping part of the entry's durable identity.

A domain folder is a *lakelet*.
The domain supplies the namespace name.
The lakelet supplies the storage surface.

Use domain atoms as lowercase ASCII kebab-case by default.
Keep them short and stable.
A domain should earn its name by reducing collisions,
clarifying ownership,
or making a composed lake easier to navigate.
