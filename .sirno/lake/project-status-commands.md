---
desc: The command surface for summarizing the current Sirno project state.
name: Project Status Commands
category:
  - concept
  - implemented
belongs:
  - project-config
prerequisite:
  - project-config
  - cli-interface
  - mcp-interface
  - lake
  - tide
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
MCP status accepts `show: summary | normal | full`.
Omitting `show` selects `summary`.
Summary status returns `ok`, project paths, entry and structural-field counts,
blocker counts, and a short message.
Normal status adds check policy, Tide summary, and default mist status.
Full status adds structural link policy and review-mode check output.

`project-status-commands` owns project status command spelling and behavior.
`project-config` owns project configuration.
`lake`, `tide`, and `mist` own the status domains being summarized.

The *repository witnesses* for this entry should show CLI and MCP status dispatch
and the shared surface-context operation that assembles status data.
