---
name: Frost Implementation Inventory
desc: A deprecated inventory of repository surfaces that implemented Frost.
category:
  - deprecated
  - concept
belongs:
  - sirno-frost
prerequisite:
  - sirno-frost
---

This entry records the repository surfaces that implemented Frost before Anchor replaces it.

The inventory is deprecated with the Frost design.
It now records which surfaces were removed and which Frost-named Tide surfaces remain temporary.
Anchor has taken over the accepted-baseline role.

Removed Frost implementation surfaces:

| Surface | Frost responsibility |
|---|---|
| `Cargo.toml` and `Cargo.lock` | Removed the direct `eter` dependency. |
| `Sirno.toml` | Removed `[frost]` project configuration. |
| `src/frost.rs` | Removed the `SirnoFrost` facade, private storage, snapshots, artifacts, GC, commit, and checkout. |
| `src/config.rs` | Removed Frost config parsing, rendering, and path validation. |
| `src/lock.rs` | Removed Frost snapshot and checkout state from `Sirno.lock.toml`. |
| `src/surface/context.rs` | Removed Frost commands, status output, commit gates, and checkout state. |
| `src/surface/dto.rs` | Removed Frost status, commit, GC, and checkout DTOs. |
| `src/surface/output.rs` | Removed Frost and commit readiness from status output. |
| `src/surface/error.rs` | Removed Frost command errors and open-Tide tutorial errors. |
| `src/surface/cli/mod.rs` | Removed CLI Frost commands, `--frost-path`, and Frost init prompts. |
| `src/mcp.rs` | Removed Frost MCP tools and Frost-aware entry path selection. |
| `src/identifier.rs` | Removed `eter` filesystem-id conversion helpers. |
| `src/anchor.rs` | Added the tracked accepted-baseline file model that replaces Frost snapshots. |

Temporary Frost-named Tide surfaces remain:

| Surface | Temporary responsibility |
|---|---|
| `src/tide.rs` | Compares the waterline against Anchor but still reads baseline-side policy from Frost-named settings. |
| `src/entry.rs` | Still parses and renders `meta.ripple.frost` until policy names become Anchor-side. |
| `src/structural.rs` | Still carries Frost-side structural ripple settings for Tide. |
| `src/lake.rs` | Still contains read-only checkout warning text and local protection helpers. |
| `src/freeze.rs` | Still provides local filesystem protection used by entry freeze and upstream protection. |
| `src/artifact.rs` | Still documents artifact path determinism with an old Frost reference. |

The deleted `src/frost.rs` witness blocks covered the core facade and storage path.
Anchor actualization should replace the remaining Frost-named Tide policy language.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-frost](sirno-frost.md)
- belongs (from): (none)

> **Sirno generated links end.**
