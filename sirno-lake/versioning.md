---
name: Versioning
desc: Lake-wide immutable snapshots of Sirno entries through eter.
category:
  - concept
belongs:
  - sirno-frost
prerequisite:
  - storage
---

When frost is configured,
Sirno versions the `sirno` form by freezing the lake
into a separate `eter` *frost* path.
The versioned *lake* state includes flat Markdown *entries*
and lake-owned *entry artifacts* under `.artifacts`.

The versioning *entry* is the front door for *frost* behavior.
Its local refinements define the private frost path and lock file.

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
Sirno compares the latest frozen snapshot with the current lake.

The lake is always the editable working form.
The *frost* path is private storage,
conventionally `sirno-frost`.
It is not read as part of the *entry lake*,
and it must not be placed where lake discovery can treat it as *entries*.
`sirno frost move PATH` renames this path and updates `[frost].path`.
`sirno frost mv PATH` is its short form.
`Sirno.lock.toml` records the lake state relative to that *frost* path.
It contains one `[frost]` table with `status`, `generation`, `version`,
and an optional `mutable` flag.

`sirno frost init [PATH]` configures the *frost* path and records empty version `0`.
The first *frost* commit creates the first frozen snapshot.
If active *tide* policy is configured,
that first commit may surface the whole lake as a bootstrap review worklist
because the frostline is still empty.
A *frost* commit imports the selected lake *entry* set and attached artifacts.
It writes one `eter` transaction.
The transaction contains changed *entries*,
entries whose artifact manifests changed,
and lifecycle deletions.
Changed artifact bytes are written into owner entry version directories.
Unchanged live *entries* and artifacts do not receive new content files.
Unchanged *entries* remain part of the new *lake* snapshot through `eter` snapshot reads.
Unchanged artifact bytes remain available through older owner entry version directories.
All rows written by the transaction receive the same snapshot coordinate.
An *entry* row stores the artifact path manifest for that *entry* version.
That manifest records artifact existence;
missing paths are deletions at that version.
Before writing the transaction,
Sirno removes every guard-bounded generated-link region from the committed *entry* bodies.
Generated links remain a lake projection.
The frost layer keeps metadata and prose without generated navigation regions.
Frozen lake *entries* and their artifact trees must match the current frost snapshot
after lake-only state is removed.
Before writing the transaction,
Sirno also requires the active *tide* to be clear.
A successful commit returns the new `SnapshotRef`.
If the lake matches the current frozen snapshot,
the commit returns the current snapshot reference without writing a new snapshot.
If an *entry* exists in the current frozen snapshot but is absent from the lake,
the commit writes an `eter` lifecycle deletion marker for that *entry*.
After a commit,
`Sirno.lock.toml` records `status = "current"` and the committed snapshot reference.

Direct edits to the lake are working-state edits.
They become frozen versions only after a *frost* commit.
Reading interfaces without a version selector read the lake.
A version selector pairs the requested coordinate with the current *frost* generation.
It reads from the *frost* path and changes the observed *lake* state
without changing query or check semantics.

Checkout materializes one *frost* version into a lake Markdown directory.
It resolves live *entries* and artifacts at the selected `SnapshotRef`.
It renders canonical *entry* files and writes the `.artifacts` tree.
Checkout uses an explicit conflict policy.
The conservative policy writes only into an absent or empty target directory.
CLI checkout replaces managed Markdown files in the configured lake
while preserving ignored paths.
`sirno frost checkout --latest` writes the current snapshot as a mutable current *lake*.
After explicit version checkout,
`Sirno.lock.toml` records `status = "checked-out"` and the selected snapshot reference.

A normal checkout is immutable.
Sirno applies local file protection to the lake root,
managed *entry* files,
and managed artifact trees.
It also writes a visible Markdown blockquote at the start of each checked-out *entry* body
that marks the file as read-only and says not to edit it by hand.
`sirno frost checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true`.
Committing a mutable *lake* creates a new current version.
Sirno refuses to commit an immutable checkout.

Versioning is field-level in `eter` and *entry*-level in Sirno.
Artifact manifests are versioned as fields on their owner *entry* rows.
Artifact bytes are versioned as sparse files under matching entry-version directories.
Sirno may expose *entry* history, diffs, and restore operations by reading fields at successive snapshots.
It presents those results as changes to *entries* and *structural links*.
The lake *entry* schema remains unchanged.

Restoring a version is checkout followed by a later *frost* commit.
Checkout writes a snapshot back to the lake.
Committing the restored lake creates a new current frozen snapshot,
so later work stays ordered and old snapshots remain immutable.
Undo-tree branching belongs to git or another outer *repository* mechanism.
Sirno's own version line is linear.

Retention is policy.
`sirno frost gc` keeps the latest frost snapshot as the explicit live set
and lets `eter` collect rows unreachable from that snapshot.
It also removes artifact byte files that cannot serve that snapshot.
It preserves the kept snapshot's CLI-visible version coordinate.
It advances the GC generation when rows are physically removed.
The GC generation is the collision boundary for stale `SnapshotRef`s.
Any future coordinate recycling belongs to an explicit compaction or rebase operation.
Long-term retention policy remains reserved for later design.
Sirno may later keep named versions,
recent versions,
versions tied to exported reviews,
or all versions.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-frost](sirno-frost.md)
- belongs (from): (none)

> **Sirno generated links end.**
