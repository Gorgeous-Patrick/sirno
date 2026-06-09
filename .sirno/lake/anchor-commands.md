---
desc: The command surface for accepted-baseline updates.
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

Anchor commands accept the current waterline as the new baseline.
Project status reports Anchor freshness and current ripples.

## Command Surface

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `sirno anchor update` | Accepts the current lake as the new baseline. |
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
`project-status-commands` owns read-only Anchor health reporting.
`tide` owns review obligations created from Anchor differences.

The *repository witnesses* for this entry should show CLI and MCP command dispatch
and the shared surface-context operations that perform Anchor work.
