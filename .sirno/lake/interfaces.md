---
name: Interfaces
desc: The interface surface for operating on Sirno projects.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - project-config
---

An interface is an access surface over Sirno project operations.
It gives a human operator, agent, editor, or other client a stable way to request Sirno operations
without owning each domain model itself.

The interface contract has three parts:

- accept surface-specific input;
- call the shared command surface for typed request and result data;
- render results in the format expected by that surface.

The `sirno::surface` module owns the shared command surface.
Interface surfaces should depend on it instead of duplicating command behavior.
CLI text, CLI JSON, and MCP results may differ in presentation,
but they should agree on the project operation and typed result.

Default interface surfaces are:

- the CLI for human operational work;
- the MCP interface for agent-facing project work and lake-owned skill resources.

These surface entries belong under this interface neighborhood:

- `cli-interface` is the human-facing surface and its command grammar.
- `mcp-interface` is the agent-facing surface and its resource and tool surface.

The interface boundary is the surface contract above.
Domain command entries belong with the design objects they operate on.
A new interface surface should define its own local entry, structural links, and witnesses.
It may reuse command families when the shared surface fits.
It should change this entry only when the interface-surface model itself changes.
