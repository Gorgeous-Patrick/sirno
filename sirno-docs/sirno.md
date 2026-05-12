---
name: Sirno
description: The project model that gives design a nominal intermediate form.
category:
  - concept
---

Sirno is the Semantic Intermediate Representation of Nominal Objects.

It is a bidirectional compiler for design-aware programming work.
It moves between one long-form project narrative,
a store of compact named Markdown entries,
and the repository codebase.

The central problem is project memory.
Design tends to begin as a narrative, then scatter across code, tests, comments,
review threads, and the working memory of whoever touched the project last.
Sirno gives that memory a named intermediate form.
It keeps the design readable as prose while making its pieces small enough to index,
query, revise, and cite during implementation.

Sirno gives design a nominal intermediate form.
The names are readable by humans, stable for tools,
and small enough for agents to inspect without carrying the whole project in context.

Sirno maintains structure:
entry ids, metadata fields, relation fields, generated footers,
storage conventions, and witness lookup.
People, agents, and other tools still judge design quality and semantic truth.

This division is important.
Sirno does not claim to know whether an architecture is elegant,
whether a test proves the right property,
or whether a code path truly satisfies a design claim.
It gives those claims stable handles.
That makes the conversation around design more precise:
a person can ask for an entry to be realized,
a reviewer can ask for the witness of an entry,
and an agent can inspect a bounded set of related entries before editing code.

The result should feel like a project with good names for its important ideas.
The monograph gives the reader a route through the whole design.
The store gives the project durable local objects.
The codebase gives those objects consequences.
Sirno keeps the edges between those surfaces explicit.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- none
> **Sirno generated links end.**
