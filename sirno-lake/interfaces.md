---
name: Interfaces
desc: The adapter boundary for operating on Sirno project storage.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - project-config
---

An interface is an adapter over configured Sirno project storage.
It gives a human operator, agent, editor, or other client a stable way to request Sirno operations
without owning the storage model itself.

The interface contract has three parts:

- accept adapter-specific input;
- call the shared command surface for typed request and result data;
- render results in the format expected by that adapter.

The `sirno::surface` module owns the shared command surface.
Adapters should depend on it instead of duplicating command behavior.
CLI text, CLI JSON, and MCP results may differ in presentation,
but they should agree on the project operation and typed result.

Sirno currently has two primary adapters:

- the CLI for human operational work;
- the MCP interface for agent-facing project work and lake-owned skill resources.

Current interface and command-family map:

- `cli-interface` is the human adapter and its command grammar.
- `mcp-interface` is the agent adapter and its resource and tool surface.
- `project-commands` covers project setup, lake movement, Anchor, checks, and rendering.
- `entry-commands` covers entry creation, paths, artifacts, freezing, queries, ripgrep, and witnesses.
- `tide-commands` covers dependency review status and resolution commands.
- `utility-commands` covers local operator utilities such as config, entry defaults, skills, and MCP startup.

This map is a reader route, not the interface boundary itself.
A new adapter should define its own local entry, structural links, and witnesses.
It may reuse command families when the shared surface fits.
It should change this entry only when the adapter model itself changes.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [cli-interface](cli-interface.md)
  - [entry-commands](entry-commands.md)
  - [extension-system](extension-system.md)
  - [mcp-interface](mcp-interface.md)
  - [project-commands](project-commands.md)
  - [sirno-anchor](sirno-anchor.md)
  - [tide-commands](tide-commands.md)
  - [utility-commands](utility-commands.md)

> **Sirno generated links end.**
