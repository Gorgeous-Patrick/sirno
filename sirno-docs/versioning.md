---
name: Versioning
description: Lake-wide immutable snapshots of Sirno entries through eter.
category:
  - concept
clustee:
  - sirno
refiner:
  - storage-and-interfaces
witness:
---

When history is configured,
Sirno versions the `sirno` form by committing the public Markdown lake
into a separate `eter` history root.

The versioning entry is the module closure for history behavior.
Its local entries define the private history store and public lock file.

A Sirno version is an `eter` `SnapshotRef`:
a GC generation plus an `Eterator` coordinate.
It is an immutable global snapshot of the entry lake.
It identifies the whole lake state,
not a single entry revision.
The coordinate is ordered inside its generation.
Entry metadata does not store it,
and entry ids remain stable across versions.

A ripple is the named delta between two lake states.
It is the reviewable difference made visible by comparing versions,
checkout states,
or other future lake snapshots.

The public lake is always the editable working form.
The history root is private storage,
conventionally `sirno-history`.
It is not read as part of the entry lake,
and it must not be placed where lake discovery can treat it as entries.
`sirno history mv PATH` renames this root and updates `[history].path`.
`Sirno.lock` records the public lake state relative to that history root.
It contains one `[history]` table with `status`, `generation`, `version`,
and an optional `mutable` flag.

`sirno history init` configures the history root and creates the first history commit.
A history commit imports the selected public entry set and writes one `eter` transaction.
The transaction may touch one entry or many entries.
All changed rows receive the same snapshot coordinate.
Before writing the transaction,
Sirno removes every guard-bounded generated-link region from the committed entry bodies.
Generated links remain a public-lake projection.
History stores metadata and prose without generated navigation regions.
A successful commit returns the new `SnapshotRef`.
If the public lake matches the current history snapshot,
the commit returns the current snapshot reference without writing a new snapshot.
If an entry exists in the current history snapshot but is absent from the public lake,
the commit writes an `eter` lifecycle deletion marker for that entry.
After a commit,
`Sirno.lock` records `status = "current"` and the committed snapshot reference.

Direct edits to the public lake are working-state edits.
They become history only after a commit.
Reading interfaces without a version selector read the public lake.
A version selector pairs the requested coordinate with the current history generation.
It reads from the history root and changes the observed lake state
without changing query or check semantics.

Checkout materializes one history version into a public Markdown directory.
It resolves live entries at the selected `SnapshotRef` and renders canonical entry files.
Checkout uses an explicit conflict policy.
The conservative policy writes only into an absent or empty target directory.
CLI checkout replaces managed Markdown files in the configured public lake
while preserving ignored paths.
After checkout,
`Sirno.lock` records `status = "checked-out"` and the selected snapshot reference.

A normal checkout is immutable.
Sirno removes write permission from the public lake root and managed entry files.
It also writes a visible Markdown blockquote at the start of each checked-out entry body
that marks the file as read-only and says not to edit it by hand.
`sirno history checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true`.
Committing an unsafe mutable checkout creates a new current version.
Sirno refuses to commit an immutable checkout.

History is field-level in `eter` and entry-level in Sirno.
Sirno may expose entry history, diffs, and restore operations by reading fields at successive snapshots.
It presents those results as changes to entries and structural fields.
The public entry schema remains unchanged.

Restoring a version is checkout followed by a later history commit.
Checkout writes a snapshot back to the public lake.
Committing the restored public lake creates a new current history snapshot,
so later work stays ordered and old snapshots remain immutable.
Undo-tree branching belongs to git or another outer repository mechanism.
Sirno's own version line is linear.

Retention is policy.
Sirno may keep named versions,
recent versions,
versions tied to exported reviews,
or all versions.
Unkept versions can be retired and garbage-collected through `eter`
only when no retained version needs their rows.
The filesystem backend does not persist retired-snapshot state,
so Sirno must provide the live set when it performs collection.

---

> **Sirno generated links begin. Do not edit this section.**

Clustee (to):
- [sirno](sirno.md)

> **Sirno generated links end.**
