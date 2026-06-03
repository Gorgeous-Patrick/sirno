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
Sirno control files live under `.sirno/` next to `Sirno.toml`.
Git stores history for the lake and control files.

The target reservoir and mist design separates canonical storage from visible projection.
The *reservoir* is the canonical tracked lake store under `.sirno/lake`.
A *mist* selects entries from the reservoir and renders them into a *misty lake*.
A misty lake is the visible workspace for humans, agents, editors, and rendered navigation.
Explicit intake writes accepted misty-lake edits back into the reservoir.

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
The target control-file split also places active Tide reviews in `.sirno/tide.toml`
and upstream dependency pins in `.sirno/lock.toml`.
The *Sirno Control Files* entry defines that split.
The reservoir, misty lakes, Sirno control files, and *repository* artifacts remain separate surfaces
with separate ownership rules.

The storage model gives Sirno durable state without making the lake *entry* files opaque.
Interfaces operate over these configured surfaces.
They should preserve the distinction between reservoir Markdown,
misty-lake workspace files,
tracked Sirno control state,
and *repository* evidence.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [anchor-file](anchor-file.md)
  - [misty-lake](misty-lake.md)
  - [reservoir](reservoir.md)
  - [sirno-anchor](sirno-anchor.md)
  - [sirno-control-files](sirno-control-files.md)

> **Sirno generated links end.**
