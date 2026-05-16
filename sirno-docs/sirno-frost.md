---
name: Sirno Frost
desc: The private eter-backed path that freezes immutable snapshots of the public Sirno Lake.
category:
  - concept
belongs:
  - frost-versioning
refines:
  - versioning
---

Sirno Frost is the optional private `eter` storage path for immutable Sirno Lake snapshots.
The default convention is `sirno-frost`.
`[frost].path` in `Sirno.toml` names the path,
and the path must stay separate from the public Markdown *lake*.
The public *lake* remains the editable working form.
The *frost* layer is the durable snapshot substrate behind that form.

The `SirnoFrost` facade opens the configured filesystem backend
and exposes frozen data as ordinary typed Sirno *entries*.
Each *entry* is stored under its stable id.
The backend records `name`, `desc`, ordered structural metadata,
and Markdown body as typed fields.
An *entry*'s presence is represented through the `eter` lifecycle field.
This keeps versioning in the storage layer
while preserving the public *entry* schema.
Structural field order stays in Sirno's typed structural metadata,
so a Frost round trip renders the same order back to Markdown.

`sirno frost init` configures Sirno Frost when needed
and records the empty snapshot as version `0`.
It does not immediately import or commit the public *lake*.
`--frost-path PATH` chooses a non-default path.
`sirno frost move PATH` renames the configured *frost* path
and writes the new path back to `[frost].path`.
`sirno frost mv PATH` is its short form.
The move refuses to replace an existing destination.

A *frost* commit imports the selected public *entry* set.
The public directory must pass review-mode checks before any snapshot is written.
Entries carrying `frozen:` are protected public files.
Frost refuses to commit them until `sirno melt ENTRY_ID` removes the marker.
Sirno strips generated-link regions from committed bodies,
because *generated footers* are public *lake* projections.
The commit writes one `eter` transaction and returns a `SnapshotRef`.
The transaction contains only changed *entries* and lifecycle deletion markers.
Unchanged live *entries* are inherited from earlier version files at read time.
That snapshot reference names the whole committed *lake* state.
For the filesystem backend,
`Eter.lock.toml` stores the committed version boundary.
Version files above that boundary are ignored
and removed before the next write.
If the public *lake* is unchanged,
the commit returns the current snapshot reference without writing.
If a previously live *entry* is missing from the public *lake*,
the commit records an `eter` lifecycle deletion marker.

The *frost* read path reconstructs *entries* from a selected snapshot.
It can read one *entry*,
all live *entries* at the current snapshot,
or all live *entries* at a specific `SnapshotRef`.
A CLI version coordinate is paired with the current `eter` GC generation
before the snapshot is read.

Checkout materializes one frozen snapshot as Markdown files.
The conservative write policy writes only into an absent or empty target directory.
CLI checkout replaces managed Markdown files in the configured public *lake*
and preserves ignored paths.
`sirno frost checkout --latest` materializes the current snapshot as a mutable current *lake*.
Explicit version checkout writes a visible read-only blockquote
and removes write permission from the *lake* root and managed *entry* files.
`--unsafe-mutable` leaves an explicit version checkout writable.

`Sirno.lock.toml` records the public *lake* state relative to *frost*.
`status = "current"` means the public *lake* is the editable current version.
`status = "checked-out"` means the public *lake* materializes a selected frozen version.
The lock stores the snapshot generation and version,
plus `mutable = true` only for unsafe mutable checkouts.
Sirno refuses to commit an immutable checkout.
Committing a mutable checkout creates a new current snapshot.

Sirno Frost is private substrate.
Users and tools may inspect it when debugging storage,
but normal Sirno work should read and edit the public *lake*
or use version-aware Sirno interfaces.
The *witness* regions for this *entry* show the facade,
snapshot reads,
commit path,
checkout path,
seed initialization,
and deletion handling in `src/frost.rs`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning](frost-versioning.md)
- belongs (from): (none)

> **Sirno generated links end.**
