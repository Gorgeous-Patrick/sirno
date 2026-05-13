# Sirno

*Semantic Intermediate Representation of Nominal Objects*

Sirno gives design a compact named form for design-aware programming work.
It keeps project knowledge in Markdown entries that humans can read,
tools can query,
and agents can inspect before changing code.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/sirno-20260401.png" width="40%">
    <source media="(prefers-color-scheme: light)" srcset="assets/sirno-nb-20260401.png" width="40%">
    <img src="assets/sirno-nb-20260401.png" width="40%">
  </picture>
</p>

Sirno works through three forms:

- `sirno`: a configured store of named Markdown entries, often `docs/`
- `mono`: an optional configured Markdown narrative, often raised from the store
- `code`: the repository implementation form

The `sirno` store is the human-readable intermediate representation.
It is readable as documentation,
structured enough for tools,
and small enough for agents and humans to manipulate without losing the project shape.

## The Four Transforms

Sirno names four transforms between its three forms.

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
The optional `witness:` marker tells Sirno to query repository witness blocks by the
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
Repository witness blocks open with `sirno:witness:start <entry-id>` and close
with `sirno:witness:end` inside paths selected by `[code].members`.

## Narratives

A narrative records a cognitive route through concepts.
This repository keeps its first route in the store as
[`introduction`](sirno-docs/introduction.md).
It keeps its working guide as [`methodology`](sirno-docs/methodology.md).

Narratives may be materialized guides in the Sirno store.
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
`sirno-docs/` is the current design source for the project.
Start with [`sirno-docs/introduction.md`](sirno-docs/introduction.md).
Use [`sirno-docs/methodology.md`](sirno-docs/methodology.md) as the compact working guide.

The implementation is expected to expose CLI and MCP interfaces,
with possible lightweight GUI or Obsidian integration later.
Entry storage is built around `eter`.
History is optional and initialized separately with `sirno history init`.
Configured store paths can be renamed with `sirno mv` and `sirno history mv`.
`Sirno.lock` records whether the public store is current or checked out to a history version.
Repository witnesses are built around `mosaika`.
