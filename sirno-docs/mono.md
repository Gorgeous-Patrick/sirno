---
name: Mono
description: The configured monograph that carries the whole design narrative.
category:
  - concept
clustee:
  - sirno
refiner:
  - surface
---

`mono` is the configured long-form Markdown document.
The usual convention is `DESIGN.md`.

It is the project monograph:
one readable narrative for a person who wants the whole design in one sitting.

The monograph is normal Markdown outside the Sirno store.
It does not carry Sirno entry metadata.

When the store exists, the monograph becomes the raised narrative view of the current entries.
It should preserve a route through the project,
not become a directory listing of entry prose.

The monograph earns its place by sequencing ideas.
It can introduce the problem first,
then the project model,
then the schema and operations that make the model usable.
That order matters for a human reader.
Entries can be browsed in many orders,
but the monograph should make one good route feel natural.

A healthy monograph is selective.
It names the important concepts,
explains how they fit,
and leaves dense local detail in entries.
When an entry grows enough local design to interrupt the main narrative,
the monograph can summarize the entry and trust the store to carry the detail.

This makes `mono` useful both before and after lowering.
Before lowering, it may be the best statement of intended design.
After lowering, it becomes a composed view over the store.
In both cases, it should read as a document written for a person,
not as a mechanical export of every known fact.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [sirno](sirno.md)
> **Sirno generated links end.**
