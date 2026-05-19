---
name: Storage
desc: The storage surfaces that hold Sirno entries, snapshots, config, and repository evidence.
category:
  - concept
belongs:
  - sirno
---

Sirno storage is the set of *repository* surfaces that hold design knowledge and operational state.

The public Markdown *lake* is the required editable working form.
It stores flat Markdown *entries* at the *lake* root
and lake-owned *entry artifacts* under `.artifacts`.
Configured repo members are optional and enable *witness* lookup.
The private *frost* path is optional and managed through `eter`.
`eter` provides durable storage, indexing, immutable snapshots,
field history, version retirement, and garbage collection.

Storage surfaces stay distinct.
Markdown *entries* are the human-facing form.
They are easy to read, review, diff, and edit.
Entry artifacts are public *lake* state attached to those *entries*.
Sirno Frost provides the durable snapshot substrate beneath that form
without adding version fields to *entry* metadata.
Repository *witnesses* stay in *repository* artifacts,
where they can show code, tests, config, generated files, assets,
or documents that actualize an *entry* claim.

`Sirno.toml` names configured storage paths and policies.
`Sirno.lock.toml` records the public *lake*'s *frost* state when Sirno Frost is configured.
The public *lake*, private *frost* path, and *repository* artifacts remain separate surfaces
with separate ownership rules.

The storage model gives Sirno durable state without making the public *entry* files opaque.
Interfaces operate over these configured surfaces.
They should preserve the distinction between editable public Markdown,
private snapshot storage,
and *repository* evidence.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from): (none)

> **Sirno generated links end.**
