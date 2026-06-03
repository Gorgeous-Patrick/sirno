---
name: Sirno Lock
desc: The TOML file that records generated upstream dependency state.
category:
  - concept
  - implemented
belongs:
  - upstream-lake
prerequisite:
  - upstream-lake
refines:
  - versioning
---

`Sirno.lock.toml` records generated upstream dependency state that must be shared.
It is TOML and lives next to `Sirno.toml` in the current implementation.
It is a transitional surface until dependency pins move to `.sirno/lock.toml`.

When upstream lakes are configured,
the lock contains `[upstreams.DOMAIN]` tables.
Each table copies the upstream request fields from `Sirno.toml`,
stores the upstream project path and configured lake path,
and records `commit` as the exact Git object crystallized into the glacier.
Branch and tag upstreams stay pinned to that commit until explicit update.
Commit-pinned upstreams already name their resolved commit.

The target control-file split moves dependency pins to `.sirno/lock.toml`.
Anchor state belongs in `.sirno/anchor.toml`.
Active review state belongs in `.sirno/tide.toml`.

Sirno writes the lock by rendering a complete TOML file to a sibling temporary path
and renaming it into place.
A failed write leaves the previous complete lock as the public state.
