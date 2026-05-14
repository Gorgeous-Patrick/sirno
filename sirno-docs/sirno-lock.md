---
name: Sirno Lock
description: The TOML file that records the history state of the public lake.
category:
  - concept
clustee:
  - versioning
refiner:
  - versioning
witness:
---

`Sirno.lock` records the public lake's state relative to the configured history root.
It is TOML and lives next to `Sirno.toml`.
It is written only when history is configured.

The lock contains one `[history]` table.
`status = "current"` means the public lake represents the current editable history version.
`status = "checked-out"` means the public lake materializes a selected historical version.
The `generation` and `version` fields store the `eter` `SnapshotRef` for that state.
`version` is the raw `Eterator` coordinate inside the stored GC generation.

A normal checkout is immutable.
Sirno removes write permission from the public lake root and managed entry files.
It also writes a visible Markdown blockquote at the start of each checked-out entry body
that says the file is read-only and should not be edited by hand.
`sirno history checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true`.

Committing a mutable checkout writes a new current history version
and rewrites the lock to `status = "current"`.
Sirno refuses to commit an immutable checkout.

---

> **Sirno generated links begin. Do not edit this section.**

Clustee (to):
- [versioning](versioning.md)

> **Sirno generated links end.**
