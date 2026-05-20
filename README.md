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

<!-- sirno:witness:readme:begin -->
If you'd like to see for yourself how Sirno's idea works directly on Sirno,
clone the repo and start with an interactive onboarding session,
which not only installs Sirno (revertably of course lol) and sets up the MCP server for you,
but also guides you through the design of Sirno itself.

First, in an agent session inside the repo, send this prompt to install Sirno and register the MCP server:

```text
Check that you're in the sirno repository right now.
Use $sirno-bootstrap to install Sirno and setup MCP in this repository if not already done.
```

Then restart your agent so the `sirno` MCP server loads,
and start the introduction session in the fresh conversation:

```text
Start an introduction session with $sirno-narrative-session based on sirno-docs/introduction.md:
I am new to Sirno. Ask about my background and goals. Guide me through the entries I should care about.
```
<!-- sirno:witness:readme:end -->

## Setup Sirno and Quick Start

<details>
<summary>Install Sirno and drive it from your agent through the bundled MCP server. Or use the CLI directly.</summary>

<!-- sirno:witness:readme:begin -->

```sh
cargo install sirno
```

and then initialize it in your project,

```sh
sirno init
```

which opens an interactive setup plan. The full plan creates:

- a *lake* (documentation directory)
- a *frost* (history store)
- `Sirno.toml` (project level configuration)
- `Sirno.lock.toml` (project state management; don't edit or delete this)
- a few wrapper skills that teach your agent how to talk to the MCP server

Use `sirno init --all` to run the full setup without prompts.
Use `sirno init --all --claude-skills` to also link those wrappers into `.claude/skills`.
You may also recreate them separately later with their own commands.

### For LLM: in MCP

Register the stdio MCP server with your agent.

For Codex, register the server from the project root:

```sh
codex mcp add sirno -- sirno util mcp
```

For Claude Code:

```sh
claude mcp add sirno -- sirno util mcp
```

For agents that read an MCP config file directly, add an equivalent stdio server:

```json
{
  "mcpServers": {
    "sirno": {
      "command": "sirno",
      "args": ["util", "mcp"]
    }
  }
}
```

Edit the generated Markdown, then ask the agent to re-query and re-check.
Add a `sirno:witness:architecture:begin` block in code to link evidence back to the entry,
and inspect it with `sirno_entry_witness`.


### For Human: in CLI

Start a *lake* of your own:

```sh
sirno init                                   # choose config, lake, frost, skills
sirno new architecture --name "Architecture" \
  --desc "How the system is structured"      # create one entry
sirno check --mode edit                      # check while editing; dangling refs are warnings
```

Edit the generated Markdown under the lake path, then re-run `check`.
Add a `sirno:witness:architecture:begin` and `sirno:witness:architecture:end` block in code
to link evidence back to the entry.

*Frost* is the history layer for the *lake*: the lake stays mutable while you draft,
while *frost* keeps the frozen snapshots you commit, stored separately over `eter`.
`Sirno.lock.toml` records whether the *lake* is current or pinned to a frozen version.

```sh
sirno commit                                 # freeze the current lake into a new frost version
sirno checkout <version>                     # materialize a past frost version (read-only)
sirno defrost                                # check the latest version back out as writable
```

*Tide* is the design review worklist between *frost* commits.
Editing one entry ripples to its structural neighbors;
*tide* tracks those as workitems you resolve before the next commit,
so locally reasonable changes do not freeze into suboptimal design.

```sh
sirno tide status                            # entry ids that still need review
sirno tide status --show full                # full open workitems
sirno resolve <entry-id>                     # mark a workitem reviewed
sirno reset                                  # clear all tide resolutions
```

Explore an existing lake; this repository keeps its current design source in `sirno-docs/`:

```sh
sirno status                                 # project, tide, and commit readiness
sirno check --mode review                    # review boundary; dangling refs are errors
sirno query --columns id,desc                # list entry ids and desc as a table
sirno query --has category=meta              # filter by structural field target
sirno witness readme --full                  # show how this README witnesses its own intention
```

<!-- sirno:witness:readme:end -->

</details>

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

Sirno follows a few guiding principles that wishes to help the project reach its goal, quoting Melina from Elden Ring.
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

<!-- sirno:witness:readme:begin -->
## Status

Sirno currently provides a Rust library, both CLI and MCP for Markdown entry storage,
project configuration, structural checks, generated footers,
querying, lake-local ripgrep search, witness lookup over `mosaika`, entry freezing,
and optional frost snapshots over `eter`.

Future interfaces may add lightweight GUI, or Obsidian integration.

<!-- sirno:witness:readme:end -->

## License

Sirno is distributed under either the MIT license
or the Apache License, Version 2.0:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)
