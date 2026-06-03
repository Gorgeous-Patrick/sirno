---
name: Mist
desc: A query-backed projector that selects lake entries and renders a misty lake.
category:
  - concept
  - proposal
belongs:
  - sirno-lake
  - interfaces
prerequisite:
  - query
  - reservoir
refines:
  - query
---

A *mist* is a selector and projector for a Sirno reservoir.

A mist chooses entries from the reservoir through the same selection model used by query.
Text terms, structural filters, field-state filters, and later graph expansion
should live in one shared selector mechanism.
`sirno query` prints the selected entries.
A mist renders the selected entries into a *misty lake*.

The mist is the filter itself,
not the projected directory.
It names what to show,
how to lay entry addresses out,
and whether the resulting workspace is editable.
A project can keep shared mist specs under `.sirno/mist/`.
A user can keep local mist specs for personal or agent-specific workspaces.

A mist should render entries with normal entry-address layout by default.
For example,
entry address `core.design` renders as `core/design.md` inside the misty lake.
That shape preserves the old lake browsing habit while keeping canonical storage in the reservoir.

A mist may also render Sirno-owned generated navigation.
All rendered output belongs in misty lakes,
so the reservoir remains the authored source for entry metadata, prose, and artifacts.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
