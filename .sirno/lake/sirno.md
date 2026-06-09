---
name: Sirno
desc: The project model that keeps design and repository work connected through nominal entries.
category:
  - concept
---

Sirno is the Semantic Intermediate Representation of Nominal Objects.

This *lake* focuses on Sirno's general design as a tool and as a philosophical system for design work.

Perspective matters because this repository is also a Sirno-managed project.
General claims describe Sirno as the tool and project model.
`this repository` names the implementation workspace for Sirno.
`this lake` names the self-hosted Sirno Lake that describes Sirno.
Its tracked reservoir lives at `.sirno/lake`.
Its default misty workspace renders to `sirno-lake/`.

It compiles between design forms for design-aware programming work.
It keeps a canonical *lake* of compact named Markdown *entries*
and uses the *repository* as the material surface those entries shape.

The central problem is project memory.
Design tends to begin as a narrative, then scatter across code, tests, comments,
review threads, and the working memory of whoever touched the project last.
Sirno answers by naming the forms design takes:
the canonical *lake* and the material *repository*.
The `form` *entry* is the main bridge into that polarity.
The `transform` *entry* names the intentional movements across it.

Those names are readable by humans, stable for tools,
and small enough for agents to inspect, query, revise, and cite during implementation.

Sirno maintains structure:
*entry* ids, metadata fields, *structural links*, *generated footers*,
reservoir conventions, control-file conventions, and *witness* lookup.
People judge design quality and semantic truth.
Agents and tools use the handles to keep work bounded.

That boundary keeps review honest.
Sirno does not decide whether an architecture is elegant,
whether a test proves the right property,
or whether a *repository* path truly satisfies a design claim.
It gives those claims stable handles.
That makes the conversation around design more precise:
a person can ask for an *entry* to be actualized,
a reviewer can ask for the *witness* of an *entry*,
and an agent can inspect a bounded set of related *entries* before editing code.

The result should feel like a project with good names for its important ideas.
The reader spine is `form`, then `transform`.
`form` explains where project knowledge lives.
`transform` explains how work crosses those places.
Narratives give the reader routes through the design.
The *lake* gives the project durable local objects.
The *repository* gives those objects consequences and evidence.
Sirno keeps the polarity between those forms explicit.
