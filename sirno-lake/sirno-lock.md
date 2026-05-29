---
name: Sirno Lock
desc: The TOML file that records generated dependency and review state.
category:
  - concept
  - implemented
belongs:
  - anchor
  - upstream-lake
prerequisite:
  - anchor
  - upstream-lake
refines:
  - versioning
---

`Sirno.lock.toml` records generated project state that must be shared.
It is TOML and lives next to `Sirno.toml` in the current implementation.

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

The target Anchor design moves active review status to `.sirno/tide.toml`
and keeps `.sirno/lock.toml` for dependency pins only.
Anchor state belongs in `.sirno/anchor.toml`.

Sirno writes the lock by rendering a complete TOML file to a sibling temporary path
and renaming it into place.
A failed write leaves the previous complete lock as the public state.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [anchor](anchor.md)
  - [upstream-lake](upstream-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
