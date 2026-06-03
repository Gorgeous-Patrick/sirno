---
name: Sirno Lock
desc: The TOML file that records generated dependency state and temporary review state.
category:
  - concept
  - implemented
belongs:
  - sirno-anchor
  - upstream-lake
prerequisite:
  - sirno-anchor
  - upstream-lake
refines:
  - versioning
---

`Sirno.lock.toml` records generated project state that must be shared.
It is TOML and lives next to `Sirno.toml` in the current implementation.
It is a transitional surface while Anchor and Tide control files are split under `.sirno/`.

When upstream lakes are configured,
the lock contains `[upstreams.DOMAIN]` tables.
Each table copies the upstream request fields from `Sirno.toml`,
stores the upstream project path and configured lake path,
and records `commit` as the exact Git object crystallized into the glacier.
Branch and tag upstreams stay pinned to that commit until explicit update.
Commit-pinned upstreams already name their resolved commit.

When a *tide* is active,
the lock may also contain explicit tide resolutions.
Each resolution stores one `(ripple, field, direction, neighbor)` tuple
and the fingerprint of the ripple entry delta it reviewed.
Sirno derives open workitems from the current waterline and Anchor baseline.
The lock does not store a separate open worklist.

The target control-file split moves active review status to `.sirno/tide.toml`
and moves dependency pins to `.sirno/lock.toml`.
Anchor state belongs in `.sirno/anchor.toml`.

Sirno writes the lock by rendering a complete TOML file to a sibling temporary path
and renaming it into place.
A failed write leaves the previous complete lock as the public state.
