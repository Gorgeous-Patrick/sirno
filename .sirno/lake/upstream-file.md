---
name: Upstream File
desc: The tracked TOML file that records generated upstream dependency pins.
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

`.sirno/upstream.toml` records generated upstream dependency pins that must be shared.
It is TOML and lives under `.sirno/` next to the other Sirno control files.
It exists only when the project has crystallized upstream pins.

When upstream lakes are configured,
the upstream file contains `[upstreams.DOMAIN]` tables.
Each table copies the upstream request fields from `Sirno.toml`,
stores the upstream project root,
the upstream manifest path,
the optional upstream mist name,
the configured lake path,
and records `commit` as the exact Git object crystallized into the glacier.
Branch and tag upstreams stay pinned to that commit until explicit update.
Commit-pinned upstreams already name their resolved commit.

Anchor state belongs in `.sirno/anchor.toml`.
Active review state belongs in `.sirno/tide.toml`.

Sirno writes the upstream file by rendering a complete TOML file to a sibling temporary path
and renaming it into place.
A failed write leaves the previous complete upstream file as the public state.
