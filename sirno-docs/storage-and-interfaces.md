---
name: Storage And Interfaces
description: The design commitment to eter storage and CLI or MCP access surfaces.
category:
  - concept
clustee:
  - sirno
---

The entry store is managed through `eter`.
At this design stage, `eter` provides durable storage and indexing.
Versioning is reserved for later design.

Sirno exposes the store through a CLI and an MCP surface.
A lightweight GUI or Obsidian extension may later provide a direct editing experience.

Repository witnesses are managed through `mosaika`.
The entry id is the query key Sirno uses when locating marks.

The storage design separates the public Markdown surface from the durable substrate.
Markdown entries are the human-facing form.
They are easy to read, review, diff, and edit.
`eter` provides the storage and indexing foundation beneath that form,
so Sirno can grow more capable without making the entry files opaque.

The CLI is the first operational interface.
It can initialize stores, create entries, query entries, check structure,
and maintain generated link footers.
Those commands should remain plain enough to use from a terminal
and stable enough for agents and skills to call.

The MCP surface serves interactive tooling.
It can expose the same store model to agents and editors without asking them to shell out for every action.
Future GUI or Obsidian work should keep the same ownership rules:
metadata is structural,
generated footer regions are Sirno-owned,
and prose outside generated regions remains user-owned.

Witness lookup stays separate through `mosaika`.
That lets repository marks evolve with code navigation needs
while Sirno keeps the entry id as the shared nominal handle.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [sirno](sirno.md)
> **Sirno generated links end.**
