---
desc: The command surface for Anchor status, validation, and baseline updates.
name: Anchor Commands
category:
  - concept
  - implemented
belongs:
  - anchor
prerequisite:
  - anchor
  - cli-interface
  - mcp-interface
refines:
  - command-families
---

Anchor commands operate on the accepted baseline for the current Sirno Lake.
They show current ripples,
validate the accepted baseline,
and accept the current waterline as the new baseline.

## Command Surface

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `sirno anchor status` | Shows current lake ripples against `.sirno/anchor.toml`. |
| CLI | `sirno anchor check` | Validates `.sirno/anchor.toml` and compares it with the lake. |
| CLI | `sirno anchor update` | Accepts the current lake as the new baseline. |
| MCP | `sirno_anchor_status` | Returns current lake ripples against the accepted baseline. |
| MCP | `sirno_anchor_check` | Returns accepted-baseline validation and ripple state. |
| MCP | `sirno_anchor_update` | Writes the current lake as the accepted baseline. |

Anchor update runs review-mode lake checks,
derives Tide from Anchor and the current waterline,
requires every open workitem to be resolved,
writes `.sirno/anchor.toml`,
and clears obsolete Tide review state.
The first update initializes Anchor from the current lake.
Later updates require a clear Tide.

`anchor-commands` owns Anchor command spelling and behavior.
`anchor` owns the accepted-baseline subsystem.
`tide` owns review obligations created from Anchor differences.

The *repository witnesses* for this entry should show CLI and MCP command dispatch
and the shared surface-context operations that perform Anchor work.
