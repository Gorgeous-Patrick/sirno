---
name: Mist Commands
desc: The command surface for mist projection status, intake, render, and generated navigation cleanup.
category:
  - concept
  - implemented
belongs:
  - mist
prerequisite:
  - mist
  - cli-interface
  - mcp-interface
refines:
  - project-commands
---

Mist commands operate on one mist projection.
They report projection differences,
intake accepted misty-lake edits,
render selected reservoir entries,
and remove generated navigation from a projection.

## Command Surface

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `sirno mist status [MIST]` | Reports pending mist ripples and stale projection state. |
| CLI | `sirno mist intake [MIST]` | Writes accepted misty-lake entry edits back into the reservoir. |
| CLI | `sirno mist render [MIST]` | Projects selected reservoir entries and renders generated navigation. |
| CLI | `sirno mist render -n, --dry` | Reports generated navigation changes without writing files. |
| CLI | `sirno mist render --dry-run` | Alias for `sirno mist render --dry`. |
| CLI | `sirno mist render --override-json JSON` | Uses temporary structural render settings for that run. |
| CLI | `sirno mist render delete` | Removes generated navigation regions from a misty lake. |
| CLI wrapper | `sirno render ...` | Shorthand for `sirno mist render ...` on the default or active mist. |
| MCP | `sirno_mist_status` | Returns pending ripples and stale projection state. |
| MCP | `sirno_mist_intake` | Writes changed misty-lake entries back into the reservoir. |
| MCP | `sirno_mist_render` | Renders one mist projection, optionally as a dry run. |
| MCP | `sirno_mist_render_delete` | Removes generated navigation regions from one projection. |

Mist render forms print changed paths or blocking diagnostics before their summary line.
The summary counts changed entries,
not copied artifact files or projection manifest files.
The override JSON uses link relation names with edge direction lists,
such as `{"belongs":["to"]}`.
It does not write the mist spec.

`mist-commands` owns mist command spelling and behavior.
`mist` owns selection, projection settings, and render semantics.
`misty-lake` owns the projected editable workspace.
`generated-navigation` owns generated footer regions.

The *repository witnesses* for this entry should show CLI and MCP command dispatch,
argument conversion,
and the shared surface-context operations that perform mist work.
