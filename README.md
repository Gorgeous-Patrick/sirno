# Sirno

*Semantic Intermediate Representation of Nominal Objects*

<!-- sirno:witness:readme:begin -->
Sirno is a development harness for project design.
Its full name is literal: a Semantic Intermediate Representation of Nominal Objects.
It is *semantic* because entries carry design meaning.
It is *intermediate* because the *lake* sits between intent and implementation.
It is a *representation* because entries, structural links, and witnesses give that meaning form.
It is *nominal* because each design *object* is named first by an entry id.

- *Lake*: a queryable, misty collection of Markdown *entries*.
- *Entry*: a named design object whose id acts as a symbol.
- *Witness*: a link from a design claim back to the code, tests, config files, or assets it describes.
- *Tide*: the design review worklist for structural ripples across the *lake*,
  preventing locally reasonable changes from settling into suboptimal global design.

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

<!-- sirno:witness:readme:begin -->
If you've heard of *RAG*, Sirno is *human-readable RAG* that evolves naturally along with the project,
avoiding cache invalidation, infamously one of the two major headaches in computer science
(followed by naming variables, and off-by-one errors).

If you've heard of *literate programming*, Sirno gives you the same unified view of design and implementation,
but less invasive and plays well with pre-existing codebases,
because Sirno doesn't actually require them to be in the same file;
it merely adds comments and git-friendly textual trackers in the codebase.

If you've heard of *hardware-software co-design*, Sirno push forward the agenda of *documentation-codebase co-design*.
There are plenty of tools that try to generate a knowledge base "out of" a codebase,
but we envision that the code should actively help the documentation.
Instead of extracting a digest out of a codebase once and for all,
Sirno facilitates documentation and codebase co-evolution, in a holistic workspace.

If you've heard of *harness engineering*,
Sirno is a documentation harness that keeps design readable, connected, and resilient to drift.
No more vibe-coded slop that only the author can understand -- or should we say not even the author can understand?

If you've heard of *{spec/test/intent}-driven development*,
Sirno believes in all of them, and integrates these ideas and just considers them documentations of different aspects.
Details should be so abundant that the codebase can be trivially recovered from the documentation alone.

This is what we call *documentation-driven development*. Any novelty should be contained in the documentation itself.
Documentation should be able to choose its own form and representation.
If a program is the best form for expressing an algorithm or even a particular design idea, then so be it.

Sirno keeps project design in a queryable *lake* of small, named Markdown *entries*.
Each entry has a stable id, structural links, and witnesses linking back to code, tests, or assets,
so the design stays readable, connected, and resistant to drift.
<!-- sirno:witness:readme:end -->

<!-- sirno:witness:readme:begin -->
<details>
<summary>
If you'd like to see for yourself how Sirno's idea works directly on the documentation of Sirno itself...
</summary>
Clone the repo and start with an interactive onboarding session,
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
Start an introduction session with $sirno-narrative-session based on .sirno/lake/introduction.md:
I am new to Sirno. Ask about my background and goals. Guide me through the entries I should care about.
```
</details>
<!-- sirno:witness:readme:end -->

## Setup Sirno and Quick Start

<details>
<summary>Install Sirno and drive it from your agent through the bundled MCP server. Or use the CLI directly.</summary>

<!-- sirno:witness:readme:begin -->

```sh
cargo install sirno
```

Setup completion if you like:

```sh
source <(sirno util completion zsh)   # or bash, fish, powershell
# or if you want to guard it:
# command -v sirno &>/dev/null && source <(sirno util completion zsh)
```

To use it, initialize it in your project:

```sh
sirno init
```

which opens an interactive setup plan. The full plan creates:

- a *lake* (documentation directory)
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
sirno init                                   # choose config, lake, skills
sirno new architecture --name "Architecture" \
  --desc "How the system is structured"      # create one entry
sirno check --mode edit                      # check while editing; dangling refs are warnings
```

Edit the generated Markdown under the lake path, then re-run `check`.
Add a `sirno:witness:architecture:begin` and `sirno:witness:architecture:end` block in code
to link evidence back to the entry.

*Tide* is the design review worklist for structural ripples.
Editing one entry ripples to its structural neighbors;
*tide* tracks those as workitems you resolve while reviewing a design edit,
so locally reasonable changes do not settle into suboptimal design.

```sh
sirno tide status                            # entry ids that still need review
sirno tide status --show full                # full open workitems
sirno resolve <entry-id>                     # mark a workitem reviewed
sirno reset                                  # clear all tide resolutions
```

Explore an existing lake; this repository keeps its current design source in `.sirno/lake/`.
The default misty workspace renders to `sirno-lake/`.

```sh
sirno status                                 # project, check, and tide summary
sirno mist status                            # pending mist ripples and stale projection state
sirno mist intake                            # accept edited misty-lake entries into the reservoir
sirno mist render                            # render the default misty lake workspace
sirno check --mode review                    # review boundary; dangling refs are errors
sirno query --columns id,desc                # list entry ids and desc as a table
sirno query --has category=meta              # filter by structural link target
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
Use $sirno-narrative-session for an introduction session based on .sirno/lake/introduction.md.
I am new to Sirno. Ask about my background and goals. Guide me through the entries I should care about.
```
<!-- sirno:witness:readme:end -->

<!-- sirno:witness:readme:begin -->
## Status

Sirno currently provides a Rust library, both CLI and MCP for Markdown entry storage,
project configuration, structural checks, generated footers,
querying, lake-local ripgrep search, witness lookup over `mosaika`, entry freezing,
and Tide review tracking.

Future interfaces may add lightweight GUI, or Obsidian integration.

<!-- sirno:witness:readme:end -->

## License

Sirno is distributed under either the MIT license
or the Apache License, Version 2.0:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)
