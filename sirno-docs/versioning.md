---
name: Versioning
desc: Lake-wide immutable snapshots of Sirno entries through eter.
category:
  - concept
belongs:
  - frost-versioning
refines:
  - storage
---

When Sirno Frost is configured,
Sirno versions the `sirno` form by freezing the public Markdown *lake*
into a separate `eter` *frost* path.
The versioned *lake* state includes flat Markdown *entries*
and lake-owned *entry artifacts* under `.artifacts`.

The versioning *entry* is the front door for *frost* behavior.
Its local refinements define the private Sirno Frost and public lock file.

A Sirno version is an `eter` `SnapshotRef`:
a GC generation plus an `Eterator` coordinate.
It is an immutable global snapshot of the *entry lake*.
It identifies the whole *lake* state,
not a single *entry* or artifact revision.
The coordinate is ordered inside its generation.
The *entry* metadata does not store it,
and *entry* ids remain stable across versions.

A *ripple* is the named delta between two *lake* states.
It is the reviewable difference made visible by comparing versions,
checkout states,
or other future *lake* snapshots.
For the active *tide*,
Sirno compares the latest frozen snapshot with the current public *lake*.

The public *lake* is always the editable working form.
The *frost* path is private storage,
conventionally `sirno-frost`.
It is not read as part of the *entry lake*,
and it must not be placed where *lake* discovery can treat it as *entries*.
`sirno frost move PATH` renames this path and updates `[frost].path`.
`sirno frost mv PATH` is its short form.
`Sirno.lock.toml` records the public *lake* state relative to that *frost* path.
It contains one `[frost]` table with `status`, `generation`, `version`,
and an optional `mutable` flag.

`sirno frost init` configures the *frost* path and records empty version `0`.
`sirno frost init --frost-path PATH` chooses a non-default *frost* path.
The first *frost* commit creates the first frozen snapshot.
If active *tide* policy is configured,
that first commit may surface the whole public *lake* as a bootstrap review worklist
because the frostline is still empty.
A *frost* commit imports the selected public *entry* set and attached artifacts.
It writes one `eter` transaction.
The transaction contains changed *entries*, changed artifacts, and lifecycle deletions.
Unchanged live *entries* and artifacts do not receive new version files.
They remain part of the new *lake* snapshot through `eter` snapshot reads.
All rows written by the transaction receive the same snapshot coordinate.
Before writing the transaction,
Sirno removes every guard-bounded generated-link region from the committed *entry* bodies.
Generated links remain a public *lake* projection.
Sirno Frost keeps metadata and prose without generated navigation regions.
Frozen public *entries* and their artifact trees must match the current Frost snapshot
after public-only state is removed.
Before writing the transaction,
Sirno also requires the active *tide* to be clear.
A successful commit returns the new `SnapshotRef`.
If the public *lake* matches the current frozen snapshot,
the commit returns the current snapshot reference without writing a new snapshot.
If an *entry* exists in the current frozen snapshot but is absent from the public *lake*,
the commit writes an `eter` lifecycle deletion marker for that *entry*.
After a commit,
`Sirno.lock.toml` records `status = "current"` and the committed snapshot reference.

Direct edits to the public *lake* are working-state edits.
They become frozen versions only after a *frost* commit.
Reading interfaces without a version selector read the public *lake*.
A version selector pairs the requested coordinate with the current *frost* generation.
It reads from the *frost* path and changes the observed *lake* state
without changing query or check semantics.

Checkout materializes one *frost* version into a public Markdown directory.
It resolves live *entries* and artifacts at the selected `SnapshotRef`.
It renders canonical *entry* files and writes the `.artifacts` tree.
Checkout uses an explicit conflict policy.
The conservative policy writes only into an absent or empty target directory.
CLI checkout replaces managed Markdown files in the configured public *lake*
while preserving ignored paths.
`sirno frost checkout --latest` writes the current snapshot as a mutable current *lake*.
After explicit version checkout,
`Sirno.lock.toml` records `status = "checked-out"` and the selected snapshot reference.

A normal checkout is immutable.
Sirno removes write permission from the public *lake* root and managed *entry* files.
It also removes write permission from managed artifact trees.
It also writes a visible Markdown blockquote at the start of each checked-out *entry* body
that marks the file as read-only and says not to edit it by hand.
`sirno frost checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true`.
Committing a mutable *lake* creates a new current version.
Sirno refuses to commit an immutable checkout.

Versioning is field-level in `eter` and *entry*-level in Sirno.
Artifact bytes are versioned as separate Frost objects owned by an *entry* id and path.
Sirno may expose *entry* history, diffs, and restore operations by reading fields at successive snapshots.
It presents those results as changes to *entries* and *structural fields*.
The public *entry* schema remains unchanged.

Restoring a version is checkout followed by a later *frost* commit.
Checkout writes a snapshot back to the public *lake*.
Committing the restored public *lake* creates a new current frozen snapshot,
so later work stays ordered and old snapshots remain immutable.
Undo-tree branching belongs to git or another outer *repository* mechanism.
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

- belongs (to):
  - [frost-versioning](frost-versioning.md)
- belongs (from): (none)

> **Sirno generated links end.**
