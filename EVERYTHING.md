# Everything Known About Sirno Progress

This document records the current state of Sirno after the documentation reshaping session.
It is intentionally exhaustive.
It includes project decisions, discarded framings, document changes, git state,
and future work discussed during the session.

---

## Repository State

The repository is `/Users/arctic/Arc/miorin-project/sirno`.

The repository currently contains documentation, assets, Cargo metadata, and no `src/` directory.
There is no Rust implementation in this repository at this point.

`Cargo.toml` defines the package:

- package name: `sirno`
- version: `0.0.0`
- edition: `2024`

The listed dependencies are:

- `smol_str = "0.3.6"`
- `thiserror = "2"`
- `tracing = "0.1"`
- `eter = { path = "../eter" }`
- `mosaika = { path = "../mosaika" }`

`eter` is intended as the entry storage and indexing substrate.
`mosaika` is intended as the repository witness marking and lookup substrate.

The repository has visual assets:

- `assets/sirno-20260401.png`
- `assets/sirno-nb-20260401.png`
- `assets/cirno-pet/pet.json`
- `assets/cirno-pet/base.png`
- `assets/cirno-pet/spritesheet.webp`

`CLAUDE.md` is a symlink to `AGENTS.md`.
It should not be edited directly.

`README.md` used to be a symlink to `DESIGN.md`.
It has now been replaced by a real Markdown file.

---

## Session Context

The session began with a newly written `DESIGN.md` and an older `METHODOLOGY.md`.
The user said the wording and phases in `DESIGN.md` were correct,
but `METHODOLOGY.md` contained more detail that could be useful or misleading.

The first task was to read the old methodology, ask questions, and infer where the repository was going.

The discussion then established the current project model.
After that, the docs were rewritten in this order:

1. `README.md` was created from scratch.
2. `METHODOLOGY.md` was rewritten completely.
3. `DESIGN.md` was rewritten as the project monograph.
4. `DESIGN.md` was revised again to organically integrate the older satisfying design prose.
5. `DESIGN.md` was structurally reordered so the conceptual spine appears before schema-heavy material.

The user then asked for this `EVERYTHING.md` file to record all information from the session.

---

## Skill Used

The user explicitly requested use of:

`.agents/skills/readme-design-methodology-doc-writer/SKILL.md`

The skill defines the roles of:

- `README.md` as the first impression
- `DESIGN.md` as the whole design document
- `METHODOLOGY.md` as a short concentrated script of principles

The skill also says to:

- extract the current project model before rewriting
- prefer the user's latest instructions over older prose
- avoid obsolete terminology
- avoid turning optional workflows into built-in primitives
- avoid implying semantic understanding when Sirno provides structure
- document metadata and structural field syntax exactly
- keep the three public docs distinct by role

---

## Central Definition

Sirno is now defined as:

`Semantic Intermediate Representation of Nominal Objects`.

The `O` in Sirno is settled as `Objects`.
The previous expansion involving `Obligations` is obsolete.

Sirno is a bidirectional compiler for design-aware programming work.
It moves between:

- one long-form project narrative
- a store of compact named Markdown entries
- the repository codebase

Sirno gives design a nominal intermediate form.
The names are readable by humans, stable for tools,
and small enough for agents to inspect without carrying the whole project in context.

Sirno is also described as secretary-like infrastructure.
It maintains structure, ids, metadata, structural fields, generated footers, storage conventions, and witness lookup.

Sirno does not directly understand project semantics.
It does not decide whether a design is good.
It does not prove that code satisfies an entry.
It does not automatically find design-code mismatch.
It gives humans, LLMs, agents, skills, CLIs, MCP tools, and editors stable objects to inspect, connect, and revise.

---

## The Three Surfaces

Sirno works through three surfaces:

- `mono`
- `sirno`
- `code`

These are official phase/surface names in the documentation, not merely CLI shorthand.

### `mono`

`mono` is one configured Markdown document.
It is often `DESIGN.md`.

The more formal idea is the project monograph: one long-form narrative for a reader who wants the whole project in one
sitting.

The `mono` path is configurable.
`DESIGN.md` is only a convention.

There should be only one `mono` document.
If a project is too large, local details may be omitted, but the `mono` document should remain human-readable.

The `mono` document is normal Markdown outside the Sirno store.
It does not have Sirno entry metadata.

### `sirno`

`sirno` is one configured store of Markdown entries.
It is often `docs/`.

The store path is configurable.
`docs/` is only a convention.

The `sirno` store is the human-readable intermediate representation.
It is text first, structured enough for tools, and compact enough for humans and agents to manipulate locally.

Once the Sirno store exists and is maintained, it should be the preferred structured design source.

### `code`

`code` is the repository implementation surface.
It contains source files, tests, configuration, generated files, assets,
and any other artifacts that realize design decisions.

Repository artifacts can witness entries through `mosaika`.

---

## Canonical Authority

There are three relevant surfaces:

- the `mono` document
- the `sirno` store
- the working codebase

If the Sirno store is already established and maintained,
the store should generally be treated as the preferred structured source.

If the Sirno store does not exist yet, the user should specify which source is more desirable:

- the working codebase
- the `mono` document that best describes the overarching aim, intent, and design

This authority question is especially important before lower/reflect work.

---

## The Four Directions

Sirno names four directions:

- `lower`: `mono -> sirno`
- `raise`: `sirno -> mono`
- `realize`: `sirno -> code`
- `reflect`: `code -> sirno`

Direct names are also useful:

- `mono-to-sirno`
- `sirno-to-mono`
- `sirno-to-code`
- `code-to-sirno`

The direction names are conceptual operations.
They are vocabulary for humans, LLMs, skills, CLI surfaces, and MCP tools.

They are not necessarily one-shot commands that Sirno itself executes.
The user specifically leaned toward making them terminology in skills and manuals,
not necessarily actual Sirno commands.

Each direction should be able to behave on its own.

The names `lower`, `raise`, `realize`, and `reflect` are the current names.
The user liked their feel.
The exact naming may still be refined later.

### Lower

`lower` means `mono -> sirno`.

Lowering takes broad narrative design and gives it compact named form.
It splits a long narrative into entries without losing the design route that made the narrative readable.

Lowering should preserve intent.
It should not turn the monograph into a flat task list.
It should create named objects that future work can cite without retelling the whole design.

Lowering does not need to decompose an entire project uniformly.
The area being worked on needs enough entries to make the work accountable.

### Raise

`raise` means `sirno -> mono`.

Raising composes entries into a readable monograph.
It is not concatenation.

The monograph should introduce terms once, trust them afterward, and omit detail that belongs in local entries.

Raising is useful when:

- the monograph has fallen behind the store
- a reader needs the whole-project picture
- the monograph will become the next source of intent

### Realize

`realize` means `sirno -> code`.

Realizing uses entries as the named design context for implementation.

A realization step should be able to answer:

`which entry explains why this code exists?`

Sirno does not require every line of code to have a nearby entry.
Important design commitments need a named place.

Sirno itself does not implement code from prose.
It maintains the structure that lets a person or agent do that work with stable references.

### Reflect

`reflect` means `code -> sirno`.

Reflecting records durable design facts learned during implementation.

Reflect when code:

- changes a representation
- narrows an invariant
- introduces a new boundary
- invalidates an old explanation
- reveals a clearer local design than the current entries record

Reflection should happen while the code change is fresh.
The reflected entry records the durable design fact, not the incidental edit history.

---

## Entries

An entry is a Markdown file in the Sirno store.

The filename stem is the entry id.
The id is globally unique within the store and case-sensitive.

The id is the stable nominal handle.
It is used by:

- structural fields
- generated footers
- witness lookup

Entries are sized to be read in about five minutes or less.

An entry is smaller and tighter than the monograph.
It may state:

- a concept
- a category
- a clique closure
- a refinement
- an invariant
- an interface
- an implementation commitment
- a witnessable claim
- a narrative route
- another local design object with a stable name

The prose body carries the design content.
The metadata block carries structure that tools must read exactly.

Operational structure is formed only from metadata.
Markdown links in prose may help readers and external tools, but they do not define Sirno structure.

---

## Entry Ids

The entry id is the filename stem.

Ids should be globally unique within the store.
Ids are case-sensitive.

Ids can technically include path segments, but practically they will usually be filename stems due to how `eter` handles
storage.

Sirno does not currently enforce a special id character set beyond filesystem specifics.

The naming convention is lowercase ASCII kebab-case, possibly with digits.

Example:

`concept-driven-development`

---

## Entry Metadata

Each entry has a YAML metadata block.

The required fields are:

- `name`
- `description`

Both required fields are plain strings.
They are not required to be unique.
They should not use Markdown formatting.

The optional structural fields are:

- `category`
- `clustee`
- `refiner`

When present, all three are always lists.
Scalar forms are not accepted.

The optional witness marker is:

- `witness:`

`witness:` is canonical.
It has no value.
No other witness spelling or boolean form is accepted.

There is no `id` field in metadata.
The filename stem is the id.

There is no separate `meta` metadata field.
`meta` is a category id.

Example metadata:

```yaml
---
name: Witness
description: An entry whose claim is evidenced by repository artifacts.
category:
  - concept
witness:
---
```

---

## Categories

Categories are entries.

An entry uses `category` to classify itself by other entries.

Sirno should not have a hard-coded closed set of entry kinds.
Users can define categories as ordinary entries.

The initialized entries `concept` and `narrative` are ordinary entries.
They are created by `init`.
They are not privileged built-ins.

The category id `meta` can classify entries that define categories.
For example, `concept` and `narrative` may include `meta` in their own `category` field.

There is no separate `meta` field.
The earlier idea of a boolean `meta: true` field was rejected.

The `locked` field exists only as future work.
It is left undesigned for now.

---

## Concepts

A concept is an entry that gives a name to compressed project knowledge.

Concepts may cover:

- design intention
- algorithmic detail
- local vocabulary
- behavior
- behavioral specification
- test rationale
- reasons shared by several decisions

Concept-driven development is central to Sirno.

The old design text introduced this through compression:

- compression saves bandwidth
- compression reduces communication overhead
- compression scales understanding across codebase evolution and time
- LZ77 uses an adaptive dictionary to replace repeated data with compact references
- Sirno gives project knowledge a similar dictionary

Concepts serve three roles:

- They cluster behavioral specifications under one named object.
- They keep intent portable across levels of detail.
- They organize tests so that properties and constraints become easier to see.

Concepts let humans and agents refer to a bundle of meaning without restating it.

The `concept` entry is ordinary.
It is created by `init`.
It is not privileged by the system.

---

## Narratives

A narrative records a cognitive route through concepts.

The monograph is the primary narrative surface.
It is outside the store and remains normal Markdown.

Materialized narratives may also live in the Sirno store as guides.
They can state prerequisites, choose a base language, and refer to concept entries at the end or along the way.

Prerequisites and base-language choices belong in prose.
They are not mechanically tracked metadata.

Interactive narratives may be generated ephemerally by skills.
They read the Sirno store, ask positioning questions, observe responses,
and generate the next paragraph or quiz from the reader's current state.

The generated narrative is ephemeral.
Canonical knowledge remains in entries and metadata.
The narrative provides a reading interface for onboarding and knowledge transfer.

The `narrative` entry is ordinary.
It is created by `init`.
It is not privileged by the system.

---

## Clustee And Clique Closure

`clustee` is an organizational field.

A `clustee` field belongs to a clique member.
It names a clique closure entry.

Tags, scopes, namespaces, and domains approximate this same structure: a named clique of related entries.

The named clique is itself an entry.
This named entry is called the clique closure.

The clique closure gives the shared subject, local vocabulary, or design neighborhood a name
and a place for explanation.

`clustee` is purely organizational.
It does not carry hidden semantics.

The mechanism can describe a two-member clique by using a clique closure.
In that case, the closure entry records why the two members belong together.
There is no extra mechanism for that case.

---

## Refiner And Refinement

`refiner` points from a more specific entry to one or more broader entries.

Refinement unfolds a high-level idea into lower-level design, implementation, and tests.

The refined entry keeps the meaning of the concept intact while making its consequences local and concrete.

A refinement chain is a path of increasing specificity.
It starts from a compressed concept and ends near repository text.
It preserves the reason that a local decision exists.

If the programming language is expressive and clean enough that the logic is clearest in code,
the final step of refinement may be a Markdown code block.

The `refiner` field is list-valued.
An entry may refine several broader entries.

Refinement does not need to form a tree.

---

## Witness

`witness:` marks an entry whose claim is evidenced in the repository.

Witness is the current name for what older methodology called grounding.
The word `grounding` should not be used as current terminology.

Sirno queries witnesses through `mosaika` by entry id.
The entry id is the query key.

Sirno does not store a separate witness query in entry metadata.

A witness may be:

- source code
- tests
- configuration
- generated files
- assets
- any repository artifact that `mosaika` can mark and query

When an entry describes behavior, representation, or invariant,
the witness is the concrete repository text or artifact against which the claim can be checked.

A test may witness an entry when the test itself is the relevant code.

The entry body may explain how to search for, access, or interpret something in the codebase.
That is fallback guidance when `mosaika` cannot do its best.

The structural convention remains:

- `witness:` marker
- entry id as query key

---

## Generated Footers

Sirno may generate and maintain a footer at the bottom of entries.

The footer is bounded by sentinels.
The sentinels state that Sirno owns the region.
Humans and other tools should leave the region untouched.

The footer format is configurable.
It may use:

- ordinary Markdown links
- Obsidian-style links

The footer is not the source of Sirno structure.
It reflects metadata-derived structure for external tools that prefer links.

Sirno owns only the generated footer region, not the surrounding prose.

---

## Validation And Checks

Sirno checks structure.
It does not check semantic truth.

Sirno validates references lazily.

During edit work, dangling `category`, `clustee`, and `refiner` ids may be warnings.

During an explicit check, dangling ids are errors.

Witness validity is checked only during explicit checks.
Witness validity is not checked during ordinary edits.

Structural checks include:

- required metadata fields
- accepted field shapes
- reference existence
- generated footer boundaries
- witness lookup validity when requested

Checks should be light during editing and strict at review boundaries.

---

## Planning

Planning is not a core Sirno primitive.

Persistent planning is a use that LLMs, agents, or Sirno-provided skills may build on top of Sirno.

A Sirno-provided skill may represent a worklist as entries.
That worklist can use categories, refiners, clustees, and witnesses like any other entry set.

The planning artifact is a use of Sirno.
It is not part of Sirno's core ontology.

---

## Removed Or Rejected Framing

The old `obligation` field is removed.

Obligations should not appear as a first-class field or current methodology primitive.

The old expansion involving `Nominal Obligations` is obsolete.
Sirno now means `Semantic Intermediate Representation of Nominal Objects`.

The old term `grounding` is replaced by `witness`.

Sirno should avoid frequent use of the word `graph`,
because `edge` is confusing when there are multiple named structural fields:

- `category`
- `clustee`
- `refiner`
- `witness`

The older phrase `knowledge graph` should not be used as the primary current description.

Sirno should not be described as an automatic drift checker.
It provides tools and structure for humans, agents, and LLMs to avoid mismatch and inspect mismatch more efficiently.

Sirno should not be described as directly inspecting or understanding content.
It is more like a secretary.

Persistent planning should not be described as built into Sirno.
It belongs in skills or workflows built on top of Sirno.

---

## Document Roles

The documentation set is now intended to have three distinct public surfaces.

### `README.md`

`README.md` is the first impression.
It defines the project compactly, shows the visual asset, names the three surfaces, names the four directions,
sketches the entry metadata shape, and points at deeper design.

`README.md` should attract and orient.
It should not carry the full design.

The current README includes:

- Sirno as a bidirectional compiler
- the `mono`, `sirno`, and `code` surfaces
- the four directions
- entry id and metadata basics
- `category`, `clustee`, `refiner`, and `witness:`
- narrative basics
- generated footers
- status and implementation expectations

`README.md` is now a real file.
It is no longer a symlink to `DESIGN.md`.

### `METHODOLOGY.md`

`METHODOLOGY.md` is the compact working guide.

It should read like a methodology or manifesto of principles, not a glossary.

It describes what a user should expect from Sirno,
what Sirno asks people and agents to do,
and where Sirno's responsibility stops.

Its current shape includes:

- what to expect from a Sirno-assisted project
- keep one monograph
- give design a nominal form
- lower before work gets local
- realize from named objects
- reflect after code changes
- raise for whole-project reading
- use concepts for compression
- use narratives for cognitive routes
- organize without hidden semantics
- witness the repository
- let Sirno maintain generated page edges
- check at review boundaries
- treat planning as a use, not a primitive
- the Sirno habit

### `DESIGN.md`

`DESIGN.md` is the monograph for the Sirno project itself.

It should be the document someone reads when they want the whole system in one sitting.

The current `DESIGN.md` should organically integrate the older satisfying prose rather than preserve it in an appendix.

Its current section order is:

1. What Is Sirno?
2. Entries
3. Concept-Driven Development
4. Narrative
5. Categories
6. Clustee Of A Clique
7. Refinement
8. Mirror Design With Witness
9. Directions
10. Metadata
11. Generated Footers
12. Storage And Interfaces
13. Checks
14. Planning
15. Project Self-Application
16. Future Work

This order was chosen so the conceptual spine arrives before the schema-heavy material.

---

## Documentation Work Completed

`README.md` was created from scratch.
It replaced the old symlink to `DESIGN.md`.

`METHODOLOGY.md` was rewritten completely.
The first rewrite was too glossary-like.
The user said it did not tell the user what to expect from Sirno.
It was rewritten again as a methodology/manifesto.

`DESIGN.md` was rewritten completely.
The first rewrite preserved the old text in a `Preserved Design Kernel` block.
The user said the older design text should be organically combined into the new form,
because the previous design was satisfying.

`DESIGN.md` was rewritten again to integrate the old prose into the main document.
The user then asked where the old lines should fit.
The answer was:

- near the beginning, but not first
- after the opening definition and entries
- before the lower implementation and schema-heavy details

The current `DESIGN.md` now follows that plan.

---

## Git History During Session

After the first documentation rewrite pass, the changes were committed.

The commit is:

`01ed60a docs: rewrite project documentation`

That commit changed:

- `DESIGN.md`
- `METHODOLOGY.md`
- `README.md`

It also changed `README.md` mode from symlink to normal file:

`mode change 120000 => 100644 README.md`

After the commit, the working tree was clean.

After the commit, the user asked to improve `DESIGN.md`.
The current `DESIGN.md` has uncommitted changes.

Immediately before this file was created, `git status --short` showed:

```text
 M DESIGN.md
```

This `EVERYTHING.md` file is itself new and uncommitted.

The first `git commit` attempt failed because the sandbox could not create `.git/index.lock`.
The commit then succeeded after escalation approval.

---

## Current Implementation Status

There is no source implementation yet.

The project has:

- design documentation
- methodology documentation
- README documentation
- assets
- Cargo metadata
- dependency declarations for `eter` and `mosaika`

No `src/` directory exists.

Implementation targets were intentionally deferred.
When asked about first implementation target, the user said:

`Wait.
Let's figure out the guidelines first.
Ask me later.`

Therefore no implementation ordering has been decided.

Possible future implementation areas discussed but not selected include:

- entry parsing and schema
- store management through `eter`
- metadata traversal
- witness lookup through `mosaika`
- generated footer management
- CLI
- MCP
- lightweight GUI
- Obsidian extension
- skills for interactive narrative
- skills for persistent planning artifacts

---

## Storage And Interfaces

Sirno will provide:

- MCP
- CLI
- potentially a lightweight GUI
- potentially an Obsidian extension

These manage entry storage through `eter`.

The codebase witness convention is ensured through `mosaika` as a library.
Sirno queries the name/id of the entry directly in the codebase.

For this design stage, `eter` is the durable storage and indexing substrate.
Versioning in `eter` is future work.

---

## Witness Backend

`mosaika` is the witness backend.

Sirno queries by entry id.

The entry body may describe manual fallback search or access behavior, but that is prose guidance,
not the primary mechanism.

---

## Future Work

Future work explicitly mentioned:

- exact direction names may still be refined
- the `locked` field is reserved but undesigned
- `eter` versioning is future work
- planning skills may use Sirno entries
- interactive narrative is likely a skill
- materialized guide narratives may be entries
- MCP, CLI, lightweight GUI, and Obsidian extension surfaces remain to be implemented

Future work not yet ordered:

- entry parser
- metadata validator
- generated footer manager
- metadata index
- `mosaika` integration
- `eter` integration
- `init` command
- creation of initial `concept` and `narrative` entries
- check command
- edit-time warning behavior
- review-boundary strict checking

---

## Current Open Design Questions

Implementation order remains open.

The exact naming of `lower`, `raise`, `realize`, and `reflect` remains open to refinement,
though the current names are accepted enough for documentation.

The shape and semantics of `locked` are intentionally undesigned.

The scope and behavior of `eter` versioning are intentionally future work.

The exact generated footer sentinel text is not yet specified.

The exact CLI and MCP command surfaces are not yet specified.

The exact way `init` creates the initial `concept` and `narrative` entries is not yet specified.

The exact Obsidian link format is configurable but not specified.

The exact Markdown link footer format is configurable but not specified.

---

## Current Design Style Preferences

The user preferred the previous `DESIGN.md` because it felt satisfying and organic.

The current `DESIGN.md` should not feel like a preserved appendix or a schema-first technical reference.

The first `DESIGN.md` rewrite preserved the previous text word-for-word in a `Preserved Design Kernel` block.
That block has since been removed.
The current intent is organic integration of the old ideas and voice, not a verbatim appendix.

The best flow is:

1. define Sirno
2. define entries
3. introduce compression and concept-driven development
4. introduce narrative
5. introduce category, clustee, refiner, and witness
6. introduce directions
7. introduce metadata and mechanical details
8. introduce storage, checks, and future work

`DESIGN.md` should be declarative, dry, precise, and readable as a single narrative.

`METHODOLOGY.md` should be forceful and behavioral.

`README.md` should be compact and attractive without being promotional.

---

## Important Repository Writing Rules

The repository's `AGENTS.md` says inline Rust documentation is the canonical documentation source once Rust code exists.

Use:

- `//!` module docs
- `///` item docs

Key understandings, new findings, invariants, and design rationale should stay close to relevant code.

Standalone documentation should not become the only canonical source for code behavior once implementation exists.

For `DESIGN.md`, the repository asks writers to evaluate before and after edits:

- Is the structure clear and logically ordered?
- Does the prose read like it was written by a knowledgeable practitioner?
- Are redundant or overlapping sections merged or reordered?

The desired `DESIGN.md` style is:

- declarative
- dry
- precise
- impersonal but not bureaucratic
- closer to concise mathematical text than a README
- simple sentence structure
- no motivational framing
- no rhetoric
- no everyday analogies unless necessary

---

## Current Working Tree After Creating This File

Before this file was created, the only uncommitted change was `DESIGN.md`.

After this file is created, the working tree includes:

- modified `DESIGN.md`
- new `EVERYTHING.md`

No tests have been run for these documentation-only edits.

The last committed documentation rewrite remains:

`01ed60a docs: rewrite project documentation`
