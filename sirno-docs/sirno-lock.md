---
name: Sirno Lock
description: The TOML file that records the Frost state of the public lake.
category:
  - concept
refines:
  - versioning
---

`Sirno.lock` records the public lake's state relative to the configured Frost root.
It is TOML and lives next to `Sirno.toml`.
It is written only when Sirno Frost is configured.

The lock contains one `[frost]` table.
`status = "current"` means the public lake represents the current editable Frost version.
`status = "checked-out"` means the public lake materializes a selected frozen version.
The `generation` and `version` fields store the `eter` `SnapshotRef` for that state.
`version` is the raw `Eterator` coordinate inside the stored GC generation.

A normal checkout is immutable.
Sirno removes write permission from the public lake root and managed entry files.
It also writes a visible Markdown blockquote at the start of each checked-out entry body
that says the file is read-only and should not be edited by hand.
`sirno frost checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true`.

Committing a mutable checkout writes a new current Frost version
and rewrites the lock to `status = "current"`.
Sirno refuses to commit an immutable checkout.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to): (none)

> **Sirno generated links end.**
