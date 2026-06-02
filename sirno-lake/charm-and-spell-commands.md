---
name: Charm And Spell Commands
desc: The command-family split between charm preparation and spell invocation.
category:
  - concept
  - proposal
belongs:
  - extension-system
  - interfaces
prerequisite:
  - charm
  - spell
  - charm-resolution
  - spell-invocation
refines:
  - extension-system
---

Charm and spell commands separate preparation from invocation.

The CLI should expose charm discovery, enablement, setup, check, build,
and cache-clean commands under `sirno charm`.
Compilation, type check, setup, and cache maintenance are charm operations.
They prepare or validate the reviewed artifact bundle.

The CLI should expose invocation and run inspection commands under `sirno spell`.
Hook invocation and direct runtime execution are spell operations.
They run the resolved script or executable.

The MCP surface may expose discovery and status.
Execution should remain a human-enabled project policy,
because MCP clients may run in contexts where silent code execution is surprising.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [extension-system](extension-system.md)
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
