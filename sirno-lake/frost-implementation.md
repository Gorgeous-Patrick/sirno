---
name: Frost Implementation Inventory
desc: A deprecated inventory of repository surfaces that currently implement Frost.
category:
  - deprecated
  - concept
belongs:
  - sirno-frost
prerequisite:
  - sirno-frost
---

This entry records the current repository surfaces that implement Frost before Anchor replaces it.

The inventory is deprecated with the Frost design.
It exists so removal work can start from a named map instead of a fresh repository search.

Current Frost implementation surfaces include:

| Surface | Frost responsibility |
|---|---|
| `Cargo.toml` | Declares the `eter` dependency. |
| `Sirno.toml` | Configures `[frost]` and current frost-side Tide policy comments. |
| `README.md` | Documents the public Frost workflow. |
| `src/frost.rs` | Implements the `SirnoFrost` facade, `eter` storage, snapshots, artifacts, GC, commit, and checkout. |
| `src/config.rs` | Parses and renders Frost config, tutorial settings, and path validation. |
| `src/lock.rs` | Stores Frost snapshot and checkout state in `Sirno.lock.toml`. |
| `src/surface/context.rs` | Wires Frost commands, status, Tide loading, commit gates, and checkout state. |
| `src/surface/dto.rs` | Defines Frost status, commit, GC, checkout, and structural policy DTOs. |
| `src/surface/output.rs` | Renders Frost and commit readiness in status output. |
| `src/surface/error.rs` | Defines Frost command errors and open-Tide tutorial text. |
| `src/surface/cli/mod.rs` | Defines CLI Frost commands, `--frost-path`, init prompts, and terminal output. |
| `src/mcp.rs` | Exposes Frost MCP tools and Frost-aware entry path selection. |
| `src/tide.rs` | Compares waterline entries against the frostline and emits Frost-side workitem sources. |
| `src/entry.rs` | Parses and renders `meta.ripple.frost` and reviewed frozen reasons. |
| `src/structural.rs` | Carries Frost-side structural ripple settings. |
| `src/lake.rs` | Applies read-only checkout warnings, freeze markers, and protected artifact-tree behavior. |
| `src/freeze.rs` | Provides local filesystem protection used by Frost checkout and entry freeze. |
| `src/artifact.rs` | Defines artifact path rules used by Frost artifact storage. |
| `src/identifier.rs` | Converts entry ids to `eter` filesystem ids. |

The existing `sirno-frost` witness blocks cover the core facade and storage path in `src/frost.rs`.
Other surfaces are named here as removal and replacement targets for Anchor actualization.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-frost](sirno-frost.md)
- belongs (from): (none)

> **Sirno generated links end.**
