---
desc: A working guide for acting inside the Sirno Lake.
name: Methodology
category:
  - concept
  - narrative
belongs:
  - narrative
prerequisite:
  - introduction
  - concept-driven-development
  - transform
  - witness
---

Sirno is a method for keeping design and implementation in conversation.

Choose perspective before writing.
Use `Sirno` for the tool and project model.
Use `a Sirno-managed project` for any project that applies Sirno.
Use `this repository` for the implementation workspace for Sirno.
Use `this lake` for the self-hosted Sirno Lake that describes Sirno.
Its tracked reservoir lives at `.sirno/lake`.
Its default misty workspace renders to `sirno-lake/`.

The method is disciplined bookkeeping.
It gives people, agents, tools, and editors the same named design objects to inspect.
It does not decide whether a design is good.
It does not prove that code satisfies an *entry*.
It makes the relevant objects easier to name, connect, revise, and witness.

Start from the *lake*.
This repository keeps its design source in `.sirno/lake`.
Read `introduction` first when you need the first route through the project.
Then follow categories, `belongs`, `prerequisite`, `refines`, and *witnesses*
to the local design.

Name the thing before the work becomes local.
An *entry* should be small enough to read in place
and durable enough to survive the edit that made it useful.
It may name a concept, structural link relation, refinement, invariant,
implementation commitment, or narrative route.
Choose the body shape for the reader.
Use paragraphs for continuous claims,
bullets for inventories, alternatives, checks, or workflows,
and simple diagrams when relationships are clearer as a picture.
The structure should help a human co-worker understand and review the entry.

Use `category` for kind.
A category target must itself be categorized by `category`.
Use `belongs` for review locality.
Use `prerequisite` for knowledge dependencies.
Use `refines` for semantic narrowing.
Use *repository witness* blocks when the *repository* contains evidence for the *entry* claim.
Leave a structural link relation out when it does not improve navigation, review, or accountability.
Run `sirno witness ENTRY_ADDRESS --full`
and read the *entry* prose for what the evidence should mean.

Use `meta` for the project's principles, vocabulary, and Sirno-facing documentation method.
A `meta` *entry* should answer how the project should be understood and developed:
guiding principles, terms, splitting rules, narrative habits, review expectations, or agent-facing guidance.

Actualize from named objects.
Before editing repository material,
read the *entries* that govern the work.
Inspect their `belongs`, `prerequisite`, `refines`, and *witnesses*.
Code, tests, README files, generated output, and design documents outside the *lake*
should be able to answer which *entry* explains an important commitment.

Internalize while the *repository* change is fresh.
Internalize when repository material changes a representation,
narrows an invariant,
introduces a boundary,
invalidates an explanation,
or reveals a clearer local design.
The internalized prose should record the durable design fact,
not narrate the whole edit.

Write narrative routes inside the *lake* when a reader needs continuity.
Treat long-form documents outside the *lake* as repo material.
They may be useful artifacts,
but their durable design claims should be internalized into *entries*,
and their published form should be actualized from the maintained *lake*.

Witness important claims.
The *witness* may be source code, tests, configuration, generated files, or assets.
Sirno queries *witnesses* by *entry address* through `mosaika`.
The *entry* states the design claim.
The *witness* block identifies the *repository* region to inspect.
The *entry* prose should briefly say what that *repository* region is expected to demonstrate.

Let Sirno maintain *generated footers*.
The generated region is bounded by sentinels and Sirno-owned.
Metadata remains the source of structural truth.
The footer exists for navigation and interoperability.

Check at review boundaries.
During editing, some structural problems can remain warnings.
At review boundaries, dangling structural link targets and *witness* blocks
that name missing *entries* should be errors.
Checks confirm structure.
They do not replace judgment about meaning.

Treat planning as a use of Sirno, not a core primitive.
A worklist can be represented as ordinary *entries* when that helps.
Those *entries* can use categories, `belongs`, `prerequisite`, `refines`, and *witnesses*
like the rest of the *lake*.

The habit is simple.
Name the thing.
Write the *entry*.
Classify it only when classification helps.
Place it in a review neighborhood when the shared subject deserves a front door.
Name prerequisites when earlier knowledge unlocks the current claim.
Refine it when broad design needs local form.
Witness it when the *repository* contains its evidence.

Sirno keeps the structure ready.
People and agents keep the meaning alive.
