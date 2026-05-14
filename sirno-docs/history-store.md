---
name: History Store
description: The private eter store that records immutable snapshots of the public Sirno Lake.
category:
  - concept
clustee:
  - versioning
refiner:
  - versioning
witness:
---

The history store is the private `eter` root for committed Sirno snapshots.
The default convention is `sirno-history`.
It is optional.
`sirno history init` adds it to a project and commits the current public lake.
`sirno history mv PATH` renames the configured history root
and writes the new path back to `[history].path`.
The move refuses to replace an existing destination.

The public lake remains the editable Markdown working form.
The history store records immutable versions of that form.
It is not read as part of the public entry lake,
and it should not live under a path that Sirno scans for entries.

A commit imports the selected public entry set into the history store.
The commit writes one `eter` transaction and returns a `SnapshotRef`.
That snapshot reference names the whole committed lake state.
Before writing the transaction,
Sirno removes generated-link regions from committed entry bodies.
If the public lake is unchanged,
the commit returns the current snapshot reference without writing.
If a previously live entry is missing from the public lake,
the commit records an `eter` lifecycle deletion marker.

Checkout reads a selected history snapshot and writes its live entries as Markdown files.
The conservative checkout policy writes only into an absent or empty target directory.
CLI checkout replaces managed Markdown files in the configured public lake
and preserves ignored paths.

`Sirno.lock` records whether the public lake is current
or checked out to a historical snapshot.
A normal checkout is made read-only through file permissions.
The checked-out entry body also starts with a visible Markdown blockquote
that says not to edit the file by hand.
`--unsafe-mutable` leaves the checkout writable and records that choice in the lock.

The history store is private substrate.
Users and tools may inspect it when debugging storage,
but normal Sirno work should read and edit the public lake
or use version-aware Sirno interfaces.

---

> **Sirno generated links begin. Do not edit this section.**

Clustee (to):
- [versioning](versioning.md)

> **Sirno generated links end.**
