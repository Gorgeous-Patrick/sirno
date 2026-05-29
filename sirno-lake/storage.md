---
name: Storage
desc: The storage surfaces that hold Sirno entries, config, control files, and repository evidence.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - form
---

Sirno storage is the set of *repository* surfaces that hold design knowledge and operational state.

The lake is the required editable working form.
It stores flat Markdown *entries* at the *lake* root
and lake-owned *entry artifacts* under `.artifacts`.
Configured repo members are optional and enable *witness* lookup.
Anchor control state lives in `.sirno/anchor.toml` next to `Sirno.toml`.
Git stores history for the lake and control files.

Storage surfaces stay distinct.
Markdown *entries* are the human-facing form.
They are easy to read, review, diff, and edit.
Entry artifacts are lake state attached to those *entries*.
Anchor records accepted fingerprints without adding version fields to *entry* metadata.
Repository *witnesses* stay in *repository* artifacts,
where they can show code, tests, config, generated files, assets,
or documents that actualize an *entry* claim.

`Sirno.toml` names configured storage paths and policies.
`.sirno/anchor.toml` records the accepted baseline.
The lake, Sirno control files, and *repository* artifacts remain separate surfaces
with separate ownership rules.

The storage model gives Sirno durable state without making the lake *entry* files opaque.
Interfaces operate over these configured surfaces.
They should preserve the distinction between editable lake Markdown,
tracked Sirno control state,
and *repository* evidence.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [anchor](anchor.md)

> **Sirno generated links end.**
