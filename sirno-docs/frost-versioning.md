---
name: Frost Versioning
desc: The review neighborhood for Sirno's Frost versioning and freezing subsystem.
category:
  - concept
---

Frost versioning is the review front door for how Sirno freezes the *lake* into immutable state.

The subsystem has several parts:
`versioning` states the lake-wide snapshot model,
`sirno-frost` is the private `eter`-backed path that holds snapshots,
`sirno-lock` records the public *lake*'s frost state,
and `entry-freeze` protects one *entry* from Frost commits.

These parts are reviewed together.
A change to the snapshot model, the frost path, the lock file, or *entry* protection
usually constrains the others, so this *entry* gives them one neighborhood.

`versioning` and `storage` remain the broader claims these parts `refines`.
This neighborhood is the separate horizontal view:
`refines` says what a part specializes,
`belongs` here says which parts are reviewed together.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from):
  - [entry-freeze](entry-freeze.md)
  - [sirno-frost](sirno-frost.md)
  - [sirno-lock](sirno-lock.md)
  - [versioning](versioning.md)

> **Sirno generated links end.**
