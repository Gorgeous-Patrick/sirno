# Sirno

*Semantic Intermediate Representation of Nominal Objects*

Design work has a familiar failure mode.
It begins as a clear explanation,
then scatters across code, tests, comments, review threads,
and the memory of whoever last touched the project.
The next person or agent has to reconstruct the design before making a responsible change.

Sirno gives that missing middle a named form.
It keeps design in a *lake* of compact Markdown *entries*:
prose small enough to read locally,
metadata exact enough to query,
and ids stable enough to cite from code, review, or automation.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/sirno-20260401.png" width="40%">
    <source media="(prefers-color-scheme: light)" srcset="assets/sirno-nb-20260401.png" width="40%">
    <img src="assets/sirno-nb-20260401.png" width="40%">
  </picture>
</p>

<!-- sirno:witness:readme:begin -->
## Our Thoughts, Our Ambitions, The Principles We Would Follow.

> "Share them with me. Your thoughts, your ambitions.
> The principles you would follow."

A tiny Melina-shaped blessing for the work:
Sirno asks a project to make its guiding ideas shareable.
<!-- sirno:witness:readme:end -->

<!-- sirno:witness:readme:begin -->
### Concept-Driven Documentation

Sirno makes documentation compressed and comprehensive through concept-driven development.
Important ideas become named *entries*.
Each *entry* is small enough to read in place,
but precise enough to connect with other entries through metadata.
Comprehensive documentation becomes a graph of durable concepts,
not one long page that every reader has to hold in memory.
<!-- sirno:witness:readme:end -->

<!-- sirno:witness:readme:begin -->
### Repository Witness

Documentation should not float away from the repository.
Sirno lets repository artifacts witness *entry* claims by entry id.
The design stays in prose,
the evidence stays in code, tests, configuration, generated files, or assets,
and the shared id lets a reviewer move between them mechanically.
<!-- sirno:witness:readme:end -->

<!-- sirno:witness:readme:begin -->
### Meta Documentation

A Sirno-managed project can document its own documentation method.
Entries categorized by `meta` define vocabulary, reader routes, splitting habits,
term style,
and local rules for how the *lake* should grow.
The documentation paradigm can live inside the project documentation itself.
<!-- sirno:witness:readme:end -->

## The Small Move

Instead of asking a reader or agent to understand the whole project at once,
Sirno gives important design objects names.

```yaml
---
name: README
description: The first-impression route that demonstrates Sirno's documentation principles.
category:
  - meta
  - narrative
belongs:
  - sirno
refines:
  - concept-driven-development
  - witness
  - meta
---
```

This is ordinary Markdown documentation.
It is also a structural object.
The filename stem is the id.
The prose explains the claim.
The metadata says what kind of object it is,
where it should be reviewed,
which broader ideas it makes concrete,
and whether repository evidence exists.

A human sees a page worth reading.
A tool sees exact fields.
An agent sees a bounded design neighborhood before it edits the repository.

## The Model

Sirno moves among three forms:

- an optional *monograph* for long-form narrative design
- a Sirno Lake of named Markdown *entries*
- the *repository* where code, tests, configuration, and assets live

It names four transforms between them:

- `lower`: `monograph -> lake`
- `realize`: `lake -> repository`
- `reflect`: `repository -> lake`
- `raise`: `lake -> monograph`

These names describe work that people, agents, tools, or skills can perform.
They are not claims that Sirno understands project semantics by itself.
Sirno maintains the structure that makes the work inspectable:
entry ids, metadata fields, structural fields, generated footers,
storage conventions, and witness lookup.

## Entries

An *entry* is a Markdown file in the Sirno Lake.
Its filename stem is its id.
Each entry has a YAML metadata block and a prose body.

The required metadata fields are `name` and `description`.
The default structural fields are `category`, `belongs`, and `refines`.
The active structural field set is configured in `Sirno.toml`.
The optional `frozen:` field marks one public entry file as read-only.

`category` classifies an entry by other entries.
Category targets are themselves entries categorized by `meta`,
so project vocabulary is documented in the *lake* rather than hidden in code.

`belongs` places an entry inside a review neighborhood.
It gives a shared subject a front door.

`refines` points from a local entry to the broader entry it makes concrete.
It records how broad design becomes implementation detail,
local policy,
or testable behavior.

`frozen:` protects one public entry file.
Use `sirno freeze ENTRY_ID` to add the marker and make the file read-only.
Use `sirno melt ENTRY_ID` to remove the marker and resume normal editing.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.
Sirno Frost refuses to commit frozen entries.

## Witnesses And Footers

Repository *witnesses* close the loop between design and implementation.
A witness block opens and closes around a repository region selected by entry id.
Rust sources can use line-comment markers.
Markdown artifacts can use hidden HTML comment markers.
Sirno uses `mosaika` to discover those regions inside paths selected by `[repo].members`.

Generated footers make the structural graph easy to follow in Markdown tools.
The footer is bounded by sentinels and owned by Sirno.
Metadata remains the source of structural truth.

## Narratives

A narrative is a route through entries for a reader.
It decides what must be understood first,
what can be named and deferred,
and where a reader should go next.

This repository is a Sirno-managed project whose design subject is Sirno itself.
That self-application is intentional.
Start with [`sirno-docs/introduction.md`](sirno-docs/introduction.md),
then use [`sirno-docs/methodology.md`](sirno-docs/methodology.md) as the working guide.
The route through the bootstrap problem lives in
[`sirno-docs/bootstrap-resolution.md`](sirno-docs/bootstrap-resolution.md).

## Try It Here

This repository keeps its current design source in `sirno-docs/`.

```sh
cargo run -- check --mode review
cargo run -- query --format id,desc --human
cargo run -- witness readme --full
```

The first command checks the *lake* as a coherent design graph.
The second lists entry ids and descriptions as a table.
The third shows how this README witnesses its own Sirno-facing intention.

## Status

Sirno currently provides a Rust library and CLI for Markdown entry storage,
project configuration, structural checks, generated footers,
querying, witness lookup, entry freezing, and optional Sirno Frost snapshots over `eter`.

Sirno Frost is initialized separately with `sirno frost init`.
`Sirno.lock.toml` records whether the public *lake* is current
or checked out to a frozen version.
Future interfaces may add MCP, lightweight GUI, or Obsidian integration.
