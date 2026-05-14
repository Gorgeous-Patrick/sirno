---
name: Sirno Frost
description: The private eter root that freezes immutable snapshots of the public Sirno Lake.
category:
  - concept
refines:
  - versioning
---

Sirno Frost is the optional private `eter` root for immutable Sirno Lake snapshots.
The default convention is `sirno-frost`.
`[frost].path` in `Sirno.toml` names the root,
and the path must stay separate from the public Markdown lake.
The public lake remains the editable working form.
Frost is the durable snapshot substrate behind that form.

The `SirnoFrost` facade opens the configured filesystem backend
and exposes frozen data as ordinary typed Sirno entries.
Each entry is stored under its stable id.
The backend records `name`, `description`, `category`, `belongs`, `refines`,
`witness`, and Markdown body as typed fields.
Entry presence is represented through the `eter` lifecycle field.
This keeps versioning in the storage layer
while preserving the public entry schema.

`sirno frost init` configures Frost when needed
and freezes the current public lake into the first snapshot.
`sirno frost mv PATH` renames the configured Frost root
and writes the new path back to `[frost].path`.
The move refuses to replace an existing destination.

A Frost commit imports the selected public entry set.
The public directory must pass review-mode checks before any snapshot is written.
Sirno strips generated-link regions from committed bodies,
because generated footers are public-lake projections.
The commit writes one `eter` transaction and returns a `SnapshotRef`.
That snapshot reference names the whole committed lake state.
If the public lake is unchanged,
the commit returns the current snapshot reference without writing.
If a previously live entry is missing from the public lake,
the commit records an `eter` lifecycle deletion marker.

Frost read operations reconstruct entries from a selected snapshot.
They can read one entry,
all live entries at the current snapshot,
or all live entries at a specific `SnapshotRef`.
A CLI version coordinate is paired with the current `eter` GC generation
before the snapshot is read.

Checkout materializes one frozen snapshot as Markdown files.
The conservative write policy writes only into an absent or empty target directory.
CLI checkout replaces managed Markdown files in the configured public lake
and preserves ignored paths.
Normal checkout writes a visible read-only blockquote
and removes write permission from the lake root and managed entry files.
`--unsafe-mutable` leaves the checkout writable.

`Sirno.lock` records the public lake state relative to Frost.
`status = "current"` means the public lake is the editable current version.
`status = "checked-out"` means the public lake materializes a selected frozen version.
The lock stores the snapshot generation and version,
plus `mutable = true` only for unsafe mutable checkouts.
Sirno refuses to commit an immutable checkout.
Committing an unsafe mutable checkout creates a new current snapshot.

Sirno Frost is private substrate.
Users and tools may inspect it when debugging storage,
but normal Sirno work should read and edit the public lake
or use version-aware Sirno interfaces.
The witness regions for this entry show the facade,
snapshot reads,
commit path,
checkout path,
seed initialization,
and deletion handling in `src/frost.rs`.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to): (none)

> **Sirno generated links end.**
