# Sirno

*Semantic Intermediate Representation of Nominal Objects*

Sirno is a bidirectional compiler for design-aware programming work.
It lowers a single long-form project narrative into compact Markdown entries,
helps users and agents realize those entries in code,
and helps code changes reflect back into the entry store.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/sirno-20260401.png" width="40%">
    <source media="(prefers-color-scheme: light)" srcset="assets/sirno-nb-20260401.png" width="40%">
    <img src="assets/sirno-nb-20260401.png" width="40%">
  </picture>
</p>

Sirno works through three surfaces:

- `mono`: one configured Markdown narrative, often `DESIGN.md`
- `sirno`: a configured store of named Markdown entries, often `docs/`
- `code`: the repository implementation surface

The `sirno` store is the human-readable intermediate representation.
It is readable as documentation,
structured enough for tools,
and small enough for agents and humans to manipulate without losing the project shape.

## The Four Directions

Sirno names four directions between its three surfaces.

- `lower`: `mono -> sirno`
- `raise`: `sirno -> mono`
- `realize`: `sirno -> code`
- `reflect`: `code -> sirno`

These names describe work that a human, an agent, or a skill can perform.
They are not promises that Sirno understands project semantics by itself.
Sirno maintains the structure that makes those movements precise:
entry ids, metadata fields, structural fields, generated footers,
storage conventions, and witness lookup.

## Entries

An entry is a Markdown file in the Sirno store.
Its filename stem is its id.
The id is globally unique inside the store and case-sensitive.

Each entry has a YAML metadata block and a prose body.
The required fields are `name` and `description`.
The optional structural fields are list-valued `category`, `clustee`, and
`refiner`.
The optional `witness:` marker tells Sirno to query repository marks by the
entry id through `mosaika`.

```yaml
---
name: Witness
description: An entry whose claim is evidenced by repository artifacts.
category:
  - concept
witness:
---
```

`category` classifies an entry by other entries.
The initialized entries `concept` and `narrative` are ordinary entries,
not privileged built-ins.

`clustee` places an entry inside a named clique closure.
The closure is also an ordinary entry.
This gives a project a documented place for shared subject matter,
local vocabulary, or design neighborhoods.

`refiner` points from a more specific entry to the entry or entries it refines.
It records how broad design becomes local design,
implementation detail,
or testable behavior.

`witness:` marks an entry whose claim is evidenced by repository artifacts.
Sirno does not store a separate witness query.
The entry id is the query key used by `mosaika`.

## Narratives

A narrative records a cognitive route through concepts.
The monograph is the primary narrative surface:
one readable document that states the project in a single pass.

Other narratives may be materialized guides in the Sirno store.
They can state prerequisites,
choose a base language,
and link to the concepts they traverse.
Interactive narratives can also be produced ephemerally by skills that read the
Sirno store and adapt the route to a reader.

## Generated Footers

Sirno may generate a managed footer at the bottom of entries.
The footer can use Markdown links or Obsidian-style links,
depending on project configuration.

The generated region is bounded by sentinels.
Those sentinels identify the region as Sirno-owned and tell humans and tools to
leave that region untouched.

## Status

This repository is the design and implementation workspace for Sirno.
`DESIGN.md` is the current monograph for the project.
`METHODOLOGY.md` is the compact working guide for the project model.

The implementation is expected to expose CLI and MCP surfaces,
with possible lightweight GUI or Obsidian integration later.
Entry storage is built around `eter`.
Repository witnesses are built around `mosaika`.
