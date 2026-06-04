---
name: Sirno Upstream
desc: The subsystem for Git-backed upstream lakes and crystallization.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - project-config
  - upstream-file
  - lake-namespace
---

*Sirno Upstream* is the subsystem for declaring Git-backed upstream lakes,
locking them to exact Git commits,
and crystallizing them into glaciers in the current lake.

The subsystem gives one handle to the operational dependency model:
upstream declarations,
the lake system formed by those declarations,
and crystallization of glaciers.

Every upstream is included through crystallization.
The resulting glacier uses the glacier domain as its entry-address prefix,
and Sirno protects the glacier files with the `managed` frozen reason.
The glacier domain is an explicit local name in `Sirno.toml`.
It has no default derived from the Git source.
It shares its lake path with implicit local lakelets,
so an unmanaged local folder blocks crystallization for the same domain.

A lake sheaf remains the composition model for the resolved addressable view.
Sirno Upstream is the operator-facing feature that produces that local view.

## Upstream Commands

| Command | Behavior |
|---|---|
| `sirno upstream add DOMAIN --git SOURCE ...` | Declares and crystallizes a Git upstream lake. |
| `sirno upstream remove DOMAIN` | Removes the declaration and managed glacier content. |
| `sirno upstream crystallize [DOMAIN]` | Crystallizes upstreams into glaciers. |
| `sirno upstream crystallize [DOMAIN] --locked` | Uses only existing locks and cache mirrors. |
| `sirno upstream update [DOMAIN]` | Refreshes upstream locks and glacier content. |
| `sirno upstream status` | Reports upstream lock, cache, glacier, and drift state. |

`sirno upstream add` accepts exactly one of `--branch NAME`, `--tag NAME`, or `--rev COMMIT`.
`--project PATH` selects the upstream project root inside the Git tree.
`--manifest PATH` selects the project config manifest relative to that root.
It defaults to `Sirno.toml`.
`--mist MIST` imports only entries selected by that mist in the upstream project.
