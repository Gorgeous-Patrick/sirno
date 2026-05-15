---
name: Sirno Lock
description: The TOML file that records the frost state of the public lake.
category:
  - concept
refines:
  - versioning
---

`Sirno.lock.toml` records the public *lake*'s state relative to the configured *frost* root.
It is TOML and lives next to `Sirno.toml`.
It is written only when Sirno Frost is configured.

The lock contains one `[frost]` table.
`status = "current"` means the public *lake* represents the current editable *frost* version.
`status = "checked-out"` means the public *lake* materializes a selected frozen version.
The `generation` and `version` fields store the `eter` `SnapshotRef` for that state.
`version` is the raw `Eterator` coordinate inside the stored GC generation.
Sirno writes the lock by rendering a complete TOML file to a sibling temporary path
and renaming it into place.
A failed write leaves the previous complete lock as the public state.

A normal checkout is immutable.
Sirno removes write permission from the public *lake* root and managed *entry* files.
It also writes a visible Markdown blockquote at the start of each checked-out *entry* body
that says the file is read-only and should not be edited by hand.
`sirno frost checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true`.

Committing a mutable checkout writes a new current *frost* version
and rewrites the lock to `status = "current"`.
Sirno refuses to commit an immutable checkout.

---

> **Sirno generated links begin. Do not edit this section.**

belongs (to): (none)

belongs (from): (none)

> **Sirno generated links end.**
