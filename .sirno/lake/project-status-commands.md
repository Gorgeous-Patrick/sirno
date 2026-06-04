---
name: Project Status Commands
desc: The command surface for summarizing the current Sirno project state.
category:
  - concept
  - implemented
belongs:
  - project-config
prerequisite:
  - project-config
  - cli-interface
  - mcp-interface
  - sirno-lake
  - sirno-tide
  - mist
refines:
  - command-families
---

Project status commands summarize the configured Sirno project as an operational dashboard.

## Command Surface

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `sirno status` | Prints the current project status. |
| CLI | `sirno st` | Short alias for `sirno status`. |
| MCP | `sirno_status` | Returns typed project status for agent-facing callers. |

Status reports:

- config path;
- reservoir path;
- entry count;
- structural link relation count;
- review-mode lake check;
- active Tide summary;
- default mist projection state.

CLI status keeps structural policy collapsed.
MCP status returns typed structural link policy, Tide state, and mist projection state.

`project-status-commands` owns project status command spelling and behavior.
`project-config` owns project configuration.
`sirno-lake`, `sirno-tide`, and `mist` own the status domains being summarized.

The *repository witnesses* for this entry should show CLI and MCP status dispatch
and the shared surface-context operation that assembles status data.
