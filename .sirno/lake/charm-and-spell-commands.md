---
name: Charm And Spell Commands
desc: The command-family split between charm preparation and spell invocation.
category:
  - concept
  - implemented
belongs:
  - extension-system
prerequisite:
  - charm
  - spell
  - charm-resolution
  - spell-invocation
refines:
  - extension-system
---

Charm and spell commands separate preparation from invocation.

The CLI exposes charm discovery, enablement, setup, check, build,
and cache-clean commands under `sirno charm`.
Compilation, type check, setup, and cache maintenance are charm operations.
They prepare or validate the reviewed artifact bundle.

The CLI exposes direct runtime execution and inspection commands under `sirno spell`.
Spell operations run the resolved script or executable.

The minimal command surface is:

- `sirno charm list`
- `sirno charm show ENTRY_ADDRESS`
- `sirno charm enable ENTRY_ADDRESS`
- `sirno charm disable ENTRY_ADDRESS`
- `sirno charm setup ENTRY_ADDRESS`
- `sirno charm check ENTRY_ADDRESS`
- `sirno charm build ENTRY_ADDRESS`
- `sirno charm clean ENTRY_ADDRESS`
- `sirno spell list`
- `sirno spell show ENTRY_ADDRESS`
- `sirno spell run ENTRY_ADDRESS`

Hook invocation commands are reserved for the later hook design.

The MCP surface may expose discovery and status.
Execution should remain a human-enabled project policy,
because MCP clients may run in contexts where silent code execution is surprising.
