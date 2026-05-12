---
name: Sirno Store
description: The configured directory of compact named design entries.
category:
  - concept
clustee:
  - sirno
refiner:
  - surface
---

The Sirno store is the configured directory of Markdown entries.
The usual convention is `docs/`.

The store is the human-readable intermediate representation:
text first, structured enough for tools,
and compact enough for humans and agents to inspect locally.

Each entry is an ordinary Markdown file with a YAML metadata block and prose body.
The filename stem is the stable id used by relations, generated footers, and witness lookup.

Once established, the store is the preferred structured design source.

The store should feel like a set of well-named design cards.
Each card has enough prose to be useful on its own,
but it also participates in a larger graph through metadata.
The graph is intentionally small:
classification, clique membership, refinement, and witnesses.
Those relations are enough to navigate without turning the store into a separate database language.

The store is also a collaboration boundary.
A person can edit an entry directly.
A CLI can check its metadata and links.
An agent can query a few related entries before changing code.
An editor can use generated footers to expose navigation.
All of those surfaces use the same filenames and metadata.

The store should avoid becoming either a glossary or a backlog.
A glossary entry may define a word without carrying design pressure.
A backlog item may describe work without preserving the concept behind it.
A Sirno entry should name durable project knowledge:
why a commitment exists,
what relation it has to broader design,
and where implementation evidence should be found when that evidence exists.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [sirno](sirno.md)
> **Sirno generated links end.**
