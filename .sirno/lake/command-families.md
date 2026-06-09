---
name: Command Families
desc: The roster of command-home entries that partition Sirno operations.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - cli-interface
  - mcp-interface
---

Command families partition Sirno operations into focused command homes.

A command-family entry owns command spelling and behavior for one operational area.
It may cover CLI commands, MCP tools, or both.
The family roster is an index for navigating command homes.
It does not own the behavior of any listed family.

| Entry | Command area |
|---|---|
| `project-status-commands` | Project status reporting. |
| `project-setup-commands` | Top-level project setup. |
| `entry-commands` | Entry files, artifacts, freeze and melt, query, search, and witnesses. |
| `lake-commands` | Lake initialization and reservoir movement. |
| `anchor-commands` | Accepted-baseline updates. |
| `mist-commands` | Mist status, intake, render, and generated navigation cleanup. |
| `upstream-commands` | Upstream declaration, crystallization, update, and status. |
| `tide-commands` | Dependency review status and resolution. |
| `utility-commands` | Local operator maintenance commands. |
| `charm-and-spell-commands` | Charm preparation and spell invocation. |

`command-families` owns only this roster
and the rule that command behavior lives in the most specific command-family entry.
`interfaces` owns the shared interface contract.
`cli-interface` and `mcp-interface` own surface-specific grammar and result presentation.
