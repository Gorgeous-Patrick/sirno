---
name: Interfaces
desc: The CLI and MCP surfaces that operate on Sirno project storage.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - project-config
---

Sirno exposes configured project storage through CLI and MCP interfaces.
The `sirno::surface` module is the shared command surface behind both adapters.
It owns typed request and result data before each adapter renders text, JSON, or MCP results.

The CLI is the human operational interface.
It initializes projects, manages entries, checks structure, renders generated links,
maintains Anchor baselines, and reviews active tide work.

The MCP interface is the agent-facing project interface.
It exposes stable grouped project tools and lake-owned skill resources.
It keeps host setup and package maintenance as explicit human CLI actions.

Interface details are split by adapter and command family.
CLI and MCP entries describe adapters.
Command entries describe shared operations exposed through those adapters.

- `cli-interface` defines command spelling, aliases, global path selection, and output conventions.
- `project-commands` defines project setup, lake movement, Anchor, checks, and rendering.
- `entry-commands` defines entry creation, paths, artifacts, freezing, queries, ripgrep, and witnesses.
- `tide-commands` defines dependency review status and resolution commands.
- `utility-commands` defines local operator utilities such as config, entry defaults, skills, and MCP startup.
- `mcp-interface` defines MCP resources, tool names, JSON behavior, and adapter ownership.

Future editing surfaces belong in `future-work` until they have a stable adapter model.

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
