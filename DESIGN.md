# Sirno

*Semantic Intermediate Representation of Nominal Objects*

Sirno is a bidirectional compiler for design-aware programming work,
moving between one long-form project narrative, a store of compact named Markdown entries, and the repository codebase.

Design needs a form that humans can read, tools can index,
and agents can manipulate without carrying a whole project in context.
Sirno gives design that form by naming it,
and the resulting names are readable by humans, stable for tools, and small enough to circulate.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/sirno-20260401.png" width="40%">
    <source media="(prefers-color-scheme: light)" srcset="assets/sirno-nb-20260401.png" width="40%">
    <img src="assets/sirno-nb-20260401.png" width="40%">
  </picture>
</p>

---

## What Is Sirno?

Sirno works through three surfaces.

`mono` is one configured Markdown document, often `DESIGN.md`,
and it serves as the project monograph:
a readable narrative for a person who wants the whole design in one sitting.

`sirno` is one configured entry store, often `docs/`,
holding named Markdown documents with metadata blocks and bodies of prose.

`code` is the repository implementation surface,
containing source files, tests, configuration, generated files, assets,
and other artifacts that realize design decisions.

The `sirno` store is the human-readable intermediate representation:
text first, structured enough for tools, and compact enough for humans and agents to inspect locally.

Before the store exists, the user chooses which surface currently carries more authority —
the working codebase, or the monograph that best describes the intended project —
and once the store exists, Sirno treats it as the structured intermediate form.

Sirno maintains structure, giving people, agents, skills, and tools stable nominal objects
through which design and implementation can be revised.
Judging design quality and proving that code satisfies a claim are left to the people and tools that use it.

---

## Entries

The `sirno` store is a set of named Markdown documents called *entries*,
each carrying a YAML metadata block and a body of prose.

The filename stem is the entry id, globally unique within the store and case-sensitive,
and it serves as the stable nominal handle used by relation fields, generated footers, and witness lookup.
In principle, ids can follow the filesystem;
in practice, they are filename stems in lowercase ASCII kebab-case, possibly with digits,
reading like `concept-driven-development`.

An entry is sized to be read in about five minutes or less,
stating a concept, category, clique closure, refinement, invariant, interface,
implementation commitment, witnessable claim, or narrative route with local prose.
The prose body carries the design content,
while the metadata block carries structure that tools must read exactly.

---

## Concept-Driven Development

Sirno is cultivated from one elementary principle: *compression*.

Spec-driven development, intent-driven development, test-driven development —
each is effective in its own way, and yet each still leaves the same piece of the puzzle untouched.
Compression saves bandwidth, reduces communication overhead,
and is what allows understanding to scale across codebase evolution and across time.
Sirno responds with *concept-driven development*.

A *concept* is a named idea that compresses knowledge:
a design intention, an algorithmic detail, a local vocabulary, a behavior, a test rationale,
or the shared reason behind several decisions.
Cognition starts from naming things,
and it is token-efficient to keep those names stable as anchors for reference and understanding.

LZ77 uses an adaptive dictionary to replace repeated data with compact references,
and Sirno gives project knowledge a similar dictionary,
except each reference remains human-readable
and each concept entry gathers the specifications, decisions, implementation notes,
and tests that share the same name.

Concepts serve three roles at once:
they cluster behavioral specifications under one named object,
they keep intent portable across levels of detail,
and they organize tests so that properties and constraints become easier to see.

The initialized `concept` entry is ordinary, created by `init` and not privileged by the system.

---

## Narrative

Concepts alone are inert. A reader follows a route.

A *narrative* records a cognitive route through concepts.
Understanding is a process rather than a state,
and a concept unfolds over time as the reader's mental model grows and refines;
narrative shapes that unfolding.

The monograph is the primary narrative surface.
It is the `mono` document — normal Markdown outside the store,
configured by path with `DESIGN.md` as the usual convention —
and it introduces terms once and trusts them afterward.

Materialized narratives may also live in the store as guides.
They state prerequisites, choose a base language,
and refer to concept entries at the end or along the way,
and these prerequisites and base-language choices belong in prose.

An *interactive narrative* presents an entry through dialogue,
asking positioning questions, observing responses,
and generating the next paragraph or quiz from the reader's current state.
The generated narrative is ephemeral;
canonical knowledge remains in entries and relations,
while the narrative serves as a reading interface for onboarding and knowledge transfer.

The initialized `narrative` entry is ordinary, created by `init` and not privileged by the system.

---

## Relations

Entries connect through their metadata,
and four named relations carry every connection that Sirno treats as structural.

- `category` classifies an entry by other entries.
- `clustee` groups an entry into a named clique.
- `refiner` points from a refinement to the broader entries it refines.
- `witness:` declares that an entry's claim is evidenced in the repository.

Each relation refers to entries by id;
the first three are list-valued fields, while `witness:` is a canonical marker without a value.
Operational structure is formed only from metadata,
and although Markdown links in prose may help readers and external tools, they do not define Sirno structure.

The next four sections introduce each relation in turn.

---

## Category

A project's vocabulary should not be fixed by the tool that records it.

A *category* relation is a metadata field of the classified entry, classifying that entry by other entries.
Categories are themselves entries, so entry kinds form an open, project-defined vocabulary.
Meta-classification reuses the same mechanism:
the category id `meta` classifies entries that themselves define categories,
and the `concept` and `narrative` entries may be categorized by `meta` in this way.

The `locked` field is reserved for later design.
It may eventually protect entries or regions that a project wants to treat as controlled.

---

## Clustee

Tags, scopes, namespaces, and domains all approximate the same structure —
a named clique of related entries — and that named clique is itself an entry.

A *clustee* relation is a metadata field of the clique member,
grouping the entry by shared subject, local vocabulary, or design neighborhood.
The clique name provides a short route into a region of the store
without changing the entries' nominal identities.

The named entry used in `clustee` is the clique closure,
an ordinary entry that gives the group a name and a place for explanation.
The mechanism can describe an undirected relation by a two-member clique closure,
whose entry records why the two members belong together.

---

## Refiner

A concept names a compressed idea. A *refinement* unfolds it.

Refinement turns a high-level idea into lower-level design, implementation, and tests;
the refined entry keeps the meaning of the concept intact
while making its consequences local and concrete.
A refinement chain is a path of increasing specificity
that starts from a compressed concept and ends near repository text,
preserving the reason that a local decision exists.
If the programming language itself is expressive and clean enough
that the design is clearest when expressed in code,
the final step of refinement may be a Markdown code block.

A *refiner* relation is a metadata field of the refined entry, pointing to the entry it refines.
The field is list-valued,
so an entry may refine several entries when the local design realizes several broader claims.

---

## Witness

Design that never meets code drifts. A *witness* relation closes the distance.

The `witness:` marker is a metadata field of the witnessed entry,
declaring that the entry's claim is evidenced in the repository.
The marker is canonical and has no value.

Sirno queries witnesses through `mosaika` by entry id.
The witness may be source code, tests, configuration, generated files, assets,
or any repository artifact that `mosaika` can mark and query,
and a test may witness an entry when the test itself is the relevant code.

Sirno uses the entry id itself as the witness query key,
which keeps the witness relation nominal and the repository marking separate from entry prose and metadata.
The entry body may describe how to search for or interpret an artifact as fallback guidance,
while the structural convention remains the marker plus the entry id.

---

## Directions

Relations are static. Work between surfaces moves.

Sirno names four directions between its surfaces:

```text
mono ─────lower───▶ sirno ────realize──▶ code
mono ◀────raise──── sirno ◀───reflect─── code
```

The direct names are also useful:
`mono-to-sirno`, `sirno-to-mono`, `sirno-to-code`, and `code-to-sirno`.
These names are conceptual operations —
vocabulary for humans, LLMs, skills, CLI surfaces, and MCP tools —
and they need not all be implemented as one-shot commands.

Lowering gives broad design compact nominal form,
splitting a long narrative into entries without losing the design route that made the narrative readable,
and it creates named objects that future work can cite without retelling the whole design.

Raising composes entries into a readable monograph rather than concatenating them:
the monograph introduces terms once, trusts them afterward,
and omits detail that belongs in local entries.

Realizing uses entries to guide implementation,
and a realization step should be able to answer which entry explains a local design commitment.
Not every line of code needs its own entry, but important commitments need a nominal place.

Reflecting records durable design facts learned during implementation.
Reflect when code changes a representation, narrows an invariant, introduces a new boundary,
invalidates an old explanation, or reveals a clearer local design than the current entries record.

---

## Metadata

Relations and directions rest on a small, exact schema.
Every entry has a YAML metadata block whose required fields are `name` and `description`, both plain strings.
The optional structural fields are `category`, `clustee`, and `refiner`,
always lists when present, and their values are entry ids.
The optional `witness:` marker is canonical and has no value;
no other witness spelling is accepted.

```yaml
---
name: Witness
description: A relation between an entry and repository artifacts.
category:
  - concept
refiner:
  - relation
witness:
---
```

Operational relations are formed only from metadata,
and Markdown links in prose may help readers and external tools without defining Sirno structure.

---

## Generated Footers

Sirno may generate and maintain a footer at the bottom of entries,
bounded by sentinels that state Sirno owns the region
and that humans and tools should leave it untouched.

The footer format is configurable,
using either ordinary Markdown links or Obsidian-style links.
The footer is a projection of metadata-derived structure,
maintained for external tools that navigate links.

---

## Storage And Interfaces

The entry store is managed through `eter`,
which at this design stage serves as the durable storage and indexing substrate;
versioning is future work.

Sirno exposes the store through a CLI and an MCP surface,
and a lightweight GUI or Obsidian extension may later provide a direct editing experience.
Repository witnesses are managed through `mosaika`,
with the entry id serving as the query key Sirno uses when locating marks.

---

## Checks

Sirno checks structure, not semantic truth.

Structural checks include required metadata fields, accepted field shapes, reference existence,
generated footer boundaries, and witness lookup validity when requested.

Sirno validates references lazily:
dangling `category`, `clustee`, and `refiner` ids may warn during edits but become errors during an explicit check,
and witness validity is checked only during an explicit check.
Checks are light during editing and strict at explicit review boundaries,
so local movement stays fast while the final state stays sound.

---

## Planning

Planning lives in skills built on top of Sirno.

Entries are durable, named, and relationally structured,
so Sirno can support persistent planning as an application of its core primitives.
A skill may use entries to represent a worklist,
treating the worklist as ordinary entries with categories, refiners, clustees, and witnesses.

---

## Future Work

The `locked` field is reserved for later design,
where it may define how entries or generated regions resist accidental edits.

`eter` versioning is reserved for later design,
and the current design depends only on durable storage and indexing.

The exact naming of the four directions may be refined;
the current names are `lower`, `raise`, `realize`, and `reflect`.

Planning skills are future work;
they may use Sirno entries to leave durable work artifacts without changing Sirno's core fields.

---

## Project Self-Application

This repository uses Sirno's own model.

`DESIGN.md` is the monograph,
and the future store will contain compact entries for the concepts, relations, interfaces,
and implementation commitments described here.
The codebase will witness those entries through `mosaika`.

The document may grow long, but it should remain ordered as one narrative.
Local details that become too dense should be lowered into entries,
and raised back only when the monograph needs them.
