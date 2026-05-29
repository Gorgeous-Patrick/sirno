---
name: Anchor
desc: The Git-backed accepted baseline for Sirno Lake review.
category:
  - implemented
  - proposal
  - concept
belongs:
  - sirno-lake
  - interfaces
  - storage
prerequisite:
  - sirno-lake
  - tide
  - structural-edge-policy
  - project-config
  - sirno-lock
refines:
  - versioning
---

Anchor is the accepted-baseline subsystem for a Sirno Lake.

Anchor records the reviewed Sirno Lake state as tracked semantic fingerprints.
It does not store history, snapshots, or old entry bodies.
Git owns history, branching, restore, retention, and destructive history operations.
Sirno owns the accepted design baseline and the Tide comparison against that baseline.

## Storage

A Sirno-managed project keeps its human config at the repository root:

```text
Sirno.toml
```

Sirno's tracked control files live under a fixed `.sirno/` directory next to `Sirno.toml`:

```text
.sirno/
  anchor.toml
  tide.toml
  lock.toml
```

`.sirno/anchor.toml` is tracked and records the accepted lake baseline.
`.sirno/tide.toml` is tracked and records active review status for the current diff.
It is deleted after Anchor accepts the lake.
`.sirno/lock.toml` is tracked and records external upstream dependency pins.
It exists only when the project has upstreams.

`Sirno.toml` stays at the root because it is the project marker and human config.
The `.sirno/` directory groups generated or semi-generated Sirno control state
without hiding the project marker.
The `.sirno/` path is fixed for the first implementation.
A fixed path keeps documentation, merge drivers, and status output simple.

## Anchor File

`.sirno/anchor.toml` stores the minimum shared state needed to compare the current lake
against the accepted lake.

```toml
schema = 1
lake = "sirno-lake"

[entries."methodology"]
fingerprint = "sha256:..."
belongs = ["narrative"]
prerequisite = ["introduction", "sirno-witness"]

[entries."entry-with-artifacts"]
fingerprint = "sha256:..."
artifact_fingerprint = "sha256:..."
```

`schema` defines both the TOML shape and the fingerprint semantics.
Schema 1 fingerprints use canonical Sirno entry rendering:
parsed metadata, deterministic structural order, normalized line endings,
and generated footer regions ignored.
The value names the digest algorithm, such as `sha256:`.

The Anchor file stores one record for each live entry.
Each record stores the entry fingerprint and the structural link fields needed for Tide.
Structural links let Sirno reconstruct Anchor-side graph edges without reading old files.
Entry-owned artifacts are represented by an owner artifact-tree fingerprint when present.

The Anchor file contains no authoritative Git object ids.
Rebase, squash, and garbage collection cannot make Anchor references dangle.
Optional Git hints may be added later,
but any hint must be non-authoritative and safe to ignore when stale.

## Tide File

`.sirno/tide.toml` stores active review status against the current Anchor.
It is target-first because a reviewer opens one reached entry and reviews it once,
even when several ripples reached that entry.

```toml
schema = 1
anchor = "sha256:..."

[[reviews]]
entry = "methodology"
entry_fingerprint = "sha256:..."
reviewer = "arctic"
reviewed_at = "2026-05-26T14:30:00Z"

[[reviews.reaches]]
ripple = "storage"
ripple_fingerprint = "sha256:..."
field = "belongs"
direction = "from"
```

A review means that the reviewer inspected `entry` at `entry_fingerprint`
and accepted the listed ripples that reached it.
A review resolves a current workitem only when all of these values still match:
the target entry id, target entry fingerprint, ripple id, ripple fingerprint,
relation field, and edge direction.
If the target entry changes, its reviews reopen.
If a ripple changes, only reviews for that ripple reopen.
Unrelated edits do not invalidate the review.

`.sirno/tide.toml` is active review evidence, not permanent history.
`anchor update` consumes it by accepting the current lake into `.sirno/anchor.toml`
and deleting `.sirno/tide.toml`.
The durable record after acceptance is the new Anchor plus the Git commit.
A later explicit archive or export command may preserve review receipts when a project wants audit history.

## Lock File

`.sirno/lock.toml` records shared pins for external inputs.
It does not store Anchor state or Tide reviews.

```toml
schema = 1

[upstreams.core]
git = "..."
branch = "main"
project = "."
lake = "sirno-lake"
commit = "..."
```

If no upstreams are configured, the lock file is absent.
The lock file remains a dependency lockfile.
Anchor remains the accepted design baseline.
Tide remains the active review state.

## Commands

Anchor commands:

| Command | Behavior |
|---|---|
| `sirno anchor status` | Shows current lake drift against `.sirno/anchor.toml`. |
| `sirno anchor update` | Accepts the current lake and deletes `.sirno/tide.toml`. |
| `sirno anchor check` | Validates the Anchor file shape and fingerprints. |

Tide commands:

| Command | Behavior |
|---|---|
| `sirno tide status` | Shows open and reviewed targets for the current Anchor comparison. |
| `sirno tide review ENTRY` | Records review status for ripples that reach `ENTRY`. |
| `sirno tide unreview ENTRY` | Removes review status for `ENTRY` or selected reaches. |
| `sirno tide reset` | Deletes `.sirno/tide.toml`. |

`anchor update` runs review-mode lake checks,
derives Tide from Anchor and the current lake,
requires every open workitem to be covered by valid reviews,
writes a new `.sirno/anchor.toml`,
and removes `.sirno/tide.toml`.

## Implementation Status

The first Anchor implementation provides `.sirno/anchor.toml`,
entry and artifact-tree fingerprints,
`sirno anchor status`,
`sirno anchor check`,
and `sirno anchor update`.
Tide compares the waterline against Anchor when the Anchor file exists.
If Anchor is absent, Tide treats the current lake as added against an empty baseline.

Temporary implementation surfaces remain while Tide is actualized:

- review resolutions still use the existing `Sirno.lock.toml` Tide table;
- structural relation entries spell the baseline-side policy as `meta.ripple.anchor`;
- merge drivers for `.sirno/anchor.toml`, `.sirno/tide.toml`, and `.sirno/lock.toml`
  are not installed yet.

These surfaces are implementation debt, not new design direction.
The target design remains tracked `.sirno/anchor.toml`,
tracked active `.sirno/tide.toml`,
and tracked dependency-only `.sirno/lock.toml`.

## Tide Model

The waterline is the current Sirno Lake.
Anchor is the accepted baseline.
A ripple is any entry-level delta between Anchor and the waterline.
Tide derives review workitems from those ripples and the configured structural policies.

Structural relation entries spell the baseline-side policy as `meta.ripple.anchor`.
Waterline edges follow the current entry graph.
Anchor edges follow the accepted entry graph stored in `.sirno/anchor.toml`.
The policy shape stays the same:
`to`, `from`, and `clique` select which neighboring entries deserve review.

## Git Behavior

Git stores the full history.
Anchor stores the accepted comparison baseline.
Tide stores active review status.

Sirno should install merge drivers for the tracked control files:

```gitattributes
.sirno/anchor.toml merge=anchor
.sirno/tide.toml merge=sirno-tide
.sirno/lock.toml merge=sirno-lock
```

The merge drivers must always write valid TOML.
They should drop reviews they cannot prove still match current fingerprints,
letting Tide reopen those obligations.
This makes rebases and merges conservative without leaving conflict markers in Sirno control files.

## Removed Snapshot Storage

Anchor keeps these responsibilities out of Sirno:

- private snapshot storage;
- snapshot commits;
- snapshot checkouts;
- snapshot garbage collection;
- Anchor-backed entry freeze checks;
- snapshot coordinates in lock state.

Entry-owned artifacts stay part of the lake state through owner artifact-tree fingerprints.
Upstream glaciers may still use managed local protection,
but the manual `reviewed` freeze reason belongs to the old snapshot design and should be removed
or redesigned as part of Anchor actualization.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
  - [sirno-lake](sirno-lake.md)
  - [storage](storage.md)
- belongs (from):
  - [entry-artifact](entry-artifact.md)
  - [sirno-lock](sirno-lock.md)
  - [sirno-tide](sirno-tide.md)
  - [versioning](versioning.md)

> **Sirno generated links end.**
