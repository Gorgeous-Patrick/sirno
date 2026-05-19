---
name: README
desc: The first-impression route that demonstrates Sirno's documentation principles.
category:
  - meta
  - narrative
belongs:
  - narrative
refines:
  - concept-driven-development
  - sirno-witness
  - meta
---

The README is the first-impression route for Sirno.
It earns the reader's attention with a precise definition,
proves the idea on Sirno itself,
then points into the *lake* for durable detail.

## Opening Definition

The README opens with a precise, compiler-flavored definition above the mascot image.
It names Sirno as a development harness for project design.
It then reads the full name literally,
glossing each word with an `It is X because` clause:
*semantic* because entries carry design meaning,
*intermediate* because the *lake* sits between intent and implementation,
*representation* because entries, structural fields, and *witnesses* give that meaning form,
and *nominal* because each design object is named first by an *entry* id.

A short feature list follows, each item led by a Sirno term:
*Lake*, *Entry*, *Witness*, *Frost*, and *Tide*.
The list says the *lake* is queryable and misty,
an *entry* is a named design object whose id acts as a symbol,
a *witness* links a design claim back to the code, tests, config files, or assets it describes,
*Frost* keeps history for frozen *lake* snapshots,
and *Tide* is the design review worklist between *frost* commits
that keeps locally reasonable changes from freezing into suboptimal design.

The definition closes with one deliberate line:
`This is the dawn of documentation-driven development.`
The line is a small literary stake, not a feature claim.
It is part of the witnessed definition because it states the README's ambition in one sentence.

The mascot image sits immediately after the definition,
so the reader meets the idea before the picture.

## TL;DR and Onboarding Invitation

A `TL;DR` follows the mascot.
Its first paragraph compresses the project model into one breath:
a queryable *lake* of small, named *entries* with stable ids, structural field edges,
and *witnesses* back to *repository* artifacts.
That paragraph stays outside the witness markers
because it restates the design rather than asserting a demonstrable capability.

The `TL;DR` then carries the onboarding invitation,
which is witnessed because it is a concrete, runnable capability claim.
The invitation tells a reader to try the idea directly on Sirno, in two prompts split by an agent restart.
The first prompt runs `$sirno-bootstrap` to install Sirno revertably and register the MCP server.
The reader then restarts the agent so the freshly registered `sirno` MCP server loads.
The second prompt, in the fresh conversation, runs `$sirno-narrative-session`
grounded in `sirno-docs/introduction.md`.
The restart is part of the claim:
the MCP server only appears after the agent reloads its connections.
Designing Sirno on Sirno is the strongest demonstration the README can offer,
so the two-prompt block earns its own witness.

## Setup and Quick Start

A collapsible `Setup Sirno and Quick Start` section holds the concrete commands.
It is one witnessed nice bit because it demonstrates Sirno's dual audience in a single place.
The section installs with `cargo install sirno`,
then `sirno init`, which creates the *lake*, the *frost* store,
`Sirno.toml`, `Sirno.lock.toml`, and the packaged skill wrappers together.

It then splits by reader:

- *For LLM: in MCP* registers the stdio MCP server,
  with explicit lines for Codex, Claude Code, and a raw MCP config file.
- *For Human: in CLI* walks `init`, `new`, and `check`,
  the *frost* cycle of `commit`, `checkout`, and `defrost`,
  the *tide* worklist of `status`, `resolve`, and `reset`,
  and read-only exploration commands ending with `sirno witness readme --full`,
  so the README shows how it witnesses its own intention.

## Motivation and Principles

The opening pressure is documentation drift:
design begins as a clear explanation,
then scatters across *repository* artifacts and the memory of whoever last touched the project.
Sirno answers by giving design a named intermediate form.
This section is titled `Minute Motivation`.
The phrase is intentionally small and literary;
it frames a brief motive, not the full design argument.

The principles section may carry a small Melina allusion from Elden Ring,
because the joke makes the invitation memorable without changing the design claim.
It demonstrates four principles, each its own witnessed block:

- Sirno makes documentation compressed and comprehensive through concept-driven development;
  important ideas become small *entries* whose metadata keeps them connected.
- Sirno lets documentation claims be witnessed by the *repository*;
  the design stays in prose while the evidence stays where it is implemented, tested, configured, or generated.
- Sirno lets a project define its own documentation paradigm through `meta` *entries*;
  the method for growing the *lake* can live inside the *lake*.
- Interactive narrative invites the reader into a task-shaped route;
  it closes the sequence with a `$sirno-narrative-session` prompt grounded in `sirno-docs/introduction.md`.

## Status and Boundary

The README ends with an honest `Status` section.
It is witnessed because it is a verifiable claim about what Sirno provides today:
a Rust library with both CLI and MCP for entry storage, configuration, structural checks,
generated footers, querying, lake-local search, witness lookup, freezing, and optional *frost* snapshots,
with only a lightweight GUI or Obsidian integration named as future work.
Witnessing this section keeps the README from overpromising as the project grows.

The README should not become the whole design document.
It gives the reader motivation, concrete commands, and a route into the *lake* for durable detail.

The *repository witnesses* for this *entry* are hidden Markdown comments in `README.md`.
They mark the opening definition, the onboarding invitation, the Quick Start section,
the motivation, the four principle sections, and the Status section.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [narrative](narrative.md)
- belongs (from): (none)

> **Sirno generated links end.**
