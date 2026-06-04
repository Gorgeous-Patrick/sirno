---
desc: The command surface for initializing, checking, and moving a Sirno Lake.
name: Lake Commands
category:
  - concept
  - implemented
belongs:
  - command-families
prerequisite:
  - lake
  - project-config
  - cli-interface
refines:
  - command-families
---

Lake commands operate on the configured Sirno Lake as a project surface.
They keep storage setup, structural checking, and path movement on the lake boundary.
Entry-level mutations stay under entry commands.
Projection work stays under mist commands.

## Command Surface

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `sirno lake init [PATH]` | Creates the project config and ordinary seed entries. |
| CLI | `sirno lake check [-m MODE]` | Checks current entry structure in edit or review mode. |
| CLI | `sirno lake move PATH` | Changes `[lake].path` and renames the configured reservoir. |
| CLI | `sirno lake mv PATH` | Short form for `sirno lake move PATH`. |
| CLI wrapper | `sirno move lake PATH` | Top-level move wrapper for the same path move. |
| CLI wrapper | `sirno mv lake PATH` | Short form for the top-level move wrapper. |
| MCP | `sirno_lake_init` | Creates the project config and ordinary seed entries. |
| MCP | `sirno_lake_check` | Checks current entry structure. |
| MCP | `sirno_lake_move` | Moves the configured reservoir path. |

`PATH` follows the project config path rules.
Relative paths resolve from the directory that contains `Sirno.toml`.
Path moves create missing destination parents and refuse existing destinations.
When the destination is inside the moved path,
Sirno stages the directory through a temporary sibling before placing it at the requested path.

`lake-commands` owns lake command spelling and behavior.
`lake` owns the conceptual entry set.
`reservoir` owns canonical storage.
`mist` owns projection commands.

The *repository witnesses* for this entry should show CLI and MCP command dispatch
and the shared surface-context operations that perform the lake work.
