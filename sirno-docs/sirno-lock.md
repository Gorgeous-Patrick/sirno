---
name: Sirno Lock
desc: The TOML file that records the frost state of the public lake.
category:
  - concept
belongs:
  - sirno-frost
prerequisite:
  - versioning
---

`Sirno.lock.toml` records the public *lake*'s state relative to the configured *frost* path.
It is TOML and lives next to `Sirno.toml`.
It is written only when Sirno Frost is configured.

The lock contains one `[frost]` table.
`status = "current"` means the public *lake* represents the current editable *frost* version.
`status = "checked-out"` means the public *lake* materializes a selected frozen version.
`sirno checkout --latest` records `status = "current"` and leaves files writable.
The `generation` and `version` fields store the `eter` `SnapshotRef` for that state.
`version` is the raw `Eterator` coordinate inside the stored GC generation.
Sirno writes the lock by rendering a complete TOML file to a sibling temporary path
and renaming it into place.
A failed write leaves the previous complete lock as the public state.

When a *tide* is active,
the lock may also contain explicit tide resolutions.
Each resolution stores one `(ripple, field, direction, neighbor)` tuple
and the fingerprint of the ripple entry delta it reviewed.
Sirno derives open workitems from the current waterline and frostline.
The lock does not store a separate open worklist.

A normal checkout is immutable.
Sirno applies local file protection to the public *lake* root,
managed *entry* files,
and managed artifact trees.
It also writes a visible Markdown blockquote at the start of each checked-out *entry* body
that says the file is read-only and should not be edited by hand.
`sirno checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true`.

Committing a mutable *lake* writes a new current *frost* version
and rewrites the lock to `status = "current"`.
Sirno refuses to commit an immutable checkout.
Successful commits clear tide resolutions.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-frost](sirno-frost.md)
- belongs (from): (none)

> **Sirno generated links end.**
