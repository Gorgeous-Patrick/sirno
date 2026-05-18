# Sirno

*Semantic Intermediate Representation of Nominal Objects*

<!-- sirno:witness:readme:begin -->
Sirno is a development harness for project design.
Its full name is literal: a Semantic Intermediate Representation of Nominal Objects.
It is *semantic* because entries carry design meaning.
It is *intermediate* because the *lake* sits between intent and implementation.
It is a *representation* because entries, structural fields, and witnesses give that meaning form.
It is *nominal* because each design *object* is named first by an entry id.

- *Lake*: a queryable, misty collection of Markdown *entries*.
- *Entry*: a named design object whose id acts as a symbol.
- *Witness*: a link from a design claim back to the code, tests, config files, or assets it describes.
- *Frost*: history for frozen *lake* snapshots.
- *Tide*: the design review worklist between *frost* commits,
  preventing locally reasonable changes from freezing into suboptimal design.

This is the dawn of documentation-driven development.
<!-- sirno:witness:readme:end -->

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/sirno-20260401.png" width="40%">
    <source media="(prefers-color-scheme: light)" srcset="assets/sirno-nb-20260401.png" width="40%">
    <img src="assets/sirno-nb-20260401.png" width="40%">
  </picture>
</p>

## TL;DR

Sirno keeps project design in a queryable *lake* of small, named Markdown *entries*.
Each entry has a stable id, structural field edges, and witnesses linking back to code, tests, or assets,
so the design stays readable, connected, and resistant to drift.

New here? Start with an interactive onboarding session:

```text
Use $sirno-narrative-session for an introduction session based on sirno-docs/introduction.md.
I am new to Sirno. Ask about my background and goals. Guide me through the entries I should care about.
```

<!-- sirno:witness:readme:begin -->
## Minute Motivation

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
<!-- sirno:witness:readme:end -->

<!-- sirno:witness:readme:begin -->
## Our Thoughts, Our Ambitions, The Principles We Would Follow.

A tiny Melina-shaped blessing for the work:
Sirno follows a few guiding principles that wishes to help the project reach its goal.
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

<!-- sirno:witness:readme:begin -->
### Interactive Narrative

Understanding is a deeply personalized journey.
An interactive narrative turns the *lake* into a route shaped around the reader's background and goals.
It chooses which entries to visit first, which details to defer,
and when to stop and assess learning retention.

Try it with our repository-local narrative-session skill:

```text
Use $sirno-narrative-session for an introduction session based on sirno-docs/introduction.md.
I am new to Sirno. Ask about my background and goals. Guide me through the entries I should care about.
```
<!-- sirno:witness:readme:end -->

## Try It Here

This repository keeps its current design source in `sirno-docs/`.

```sh
cargo run -- check --mode review
cargo run -- query --columns id,desc
cargo run -- witness readme --full
```

The first command checks the *lake* as a coherent design graph.
The second lists entry ids and `desc` values as a table.
The third shows how this README witnesses its own Sirno-facing intention.

## Status

Sirno currently provides a Rust library and CLI for Markdown entry storage,
project configuration, structural checks, generated footers,
querying, lake-local ripgrep search, witness lookup, entry freezing,
and optional Sirno Frost snapshots over `eter`.

`sirno init` initializes a new public *lake*, private Frost store,
and packaged Sirno skill wrappers together.
Use `sirno commit` and `sirno checkout` for the usual Frost snapshot cycle.
`Sirno.lock.toml` records whether the public *lake* is current
or checked out to a frozen version.
Future interfaces may add MCP, lightweight GUI, or Obsidian integration.

## License

Sirno is distributed under either the MIT license
or the Apache License, Version 2.0:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)
