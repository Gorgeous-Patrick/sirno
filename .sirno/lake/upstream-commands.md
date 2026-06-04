---
desc: The command surface for declaring, crystallizing, updating, and inspecting upstream lakes.
name: Upstream Commands
category:
  - concept
  - implemented
belongs:
  - sirno-upstream
prerequisite:
  - sirno-upstream
  - project-config
  - cli-interface
  - mcp-interface
refines:
  - command-families
---

Upstream commands operate on Sirno Upstream as the dependency surface.
They declare upstream lakes,
crystallize them into glaciers,
refresh lock state,
and report upstream drift.

## Command Surface

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `sirno upstream add DOMAIN --git SOURCE ...` | Declares and crystallizes a Git upstream lake. |
| CLI | `sirno upstream remove DOMAIN` | Removes the declaration and managed glacier content. |
| CLI | `sirno upstream crystallize [DOMAIN]` | Crystallizes upstreams into glaciers. |
| CLI | `sirno upstream crystallize [DOMAIN] --locked` | Uses only existing locks and cache mirrors. |
| CLI | `sirno upstream update [DOMAIN]` | Refreshes upstream locks and glacier content. |
| CLI | `sirno upstream status` | Reports upstream lock, cache, glacier, and drift state. |
| MCP | `sirno_upstream_add` | Declares a Git upstream and crystallizes it. |
| MCP | `sirno_upstream_remove` | Removes an upstream declaration and managed glacier. |
| MCP | `sirno_upstream_crystallize` | Crystallizes configured upstream lakes. |
| MCP | `sirno_upstream_update` | Refreshes upstream locks and glaciers. |
| MCP | `sirno_upstream_status` | Returns upstream lock, cache, and glacier status. |

`sirno upstream add` accepts exactly one of `--branch NAME`, `--tag NAME`, or `--rev COMMIT`.
`--project PATH` selects the upstream project root inside the Git tree.
`--manifest PATH` selects the project config manifest relative to that root.
It defaults to `Sirno.toml`.
`--mist MIST` imports only entries selected by that mist in the upstream project.

`upstream-commands` owns upstream command spelling and behavior.
`sirno-upstream` owns the subsystem and dependency model.
`upstream-file`, `crystallization`, and `glacier` own storage and composition details.

The *repository witnesses* for this entry should show CLI and MCP command dispatch,
argument conversion,
and the shared surface-context operations that perform upstream work.
