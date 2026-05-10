# Sirno

*Semantic Intermediate Representation of Nominal Objects*

Sirno is a documentation knowledge graph for concept-driven development and anti-drift codebase alignment.
It stores named concepts and their relations, refines broad design into local implementation,
binds those claims to repository artifacts, and requires re-examination when an upstream concept or relation changes.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/sirno-20260401.png" width="40%">
    <source media="(prefers-color-scheme: light)" srcset="assets/sirno-nb-20260401.png" width="40%">
    <img src="assets/sirno-nb-20260401.png" width="40%">
  </picture>
</p>

---

## What is Sirno?

Sirno is a set of named Markdown documents, each with a metadata block and a body of prose.
The document name is a nominal identifier, and therefore gives the document a stable canonical reference.

These documents are called *entries*.
An entry is sized to be read in about five minutes or less,
and its YAML metadata block records its full name, concise description, category, and relations.

Some entries are categorized as *concepts* or *narratives*,
and relational metadata includes *clustee*, *refiner*, and *witness*.

---

## Our Thoughts, Our Ambitions, The Principles We Would Follow

Sirno is cultivated from a series of elementary principles.

### Concept-Driven Development

You may have heard of spec-driven development, intent-driven development, or test-driven development.
These methodologies are effictive in their own ways, but they are still missing one crucial pieces of the puzzle.

*Compression*.

Compression saves bandwidth and therefore reduces communication overhead.
Compression is the key to scaling understanding across codebase evolution and across time.
In response, we propose concept-driven development.

*Concepts* are the named ideas that compress knowledge, everything from design intention to algorithmic details.
Cognition starts from naming things.
It's rather token-efficient to keep those names as the stable anchors for understanding and reference.

LZ77 uses an adaptive dictionary to replace repeated data with compact references.
Sirno gives project knowledge a similar dictionary:
each reference remains human-readable,
and a concept entry gathers the specifications, decisions, and tests that share the same name.

Reflecting on aforementioned paradigms, concepts serve three roles simultaneously:
- They cluster behavioral specifications under one named object.
- They keep intent portable across levels of detail.
- They organize tests so that properties and constraints become easier to see.

### Interactive Narrative

Browsing through a collection of documented concepts undoubtedly helps systematic understanding,
at least more systematic than chatting with a large language model or reading code alone.

Understanding is a process rather than a state.
A concept may unfold over time as the reader's mental model grows and refines.
But there's a gap: the reader's progress is not directly observable by the writer.

We all know that narrative shapes understanding and flattens learning curve.
But what we often miss is that narrative can be tailored to the reader's current understanding and needs,
which demands not only presentation but also interaction.

An *interactive narrative* presents an entry through dialogue,
asking positioning questions, observing responses,
and generating the next paragraph or quiz from the reader's current state.

The generated narrative is ephemeral;
canonical knowledge remains in entries and relations,
while the narrative provides a reading interface for onboarding and knowledge transfer.

### Clustee of a Clique

Tags, scopes, namespaces, and domains all approximate the same structure:
a named clique of related entries. Such named clique is an entry itself.

A *clustee* relation is a metadata block field of the clique member.
It groups entries by shared subject, local vocabulary, or design neighborhood,
and the clique name provides a short route into a region of the graph
without changing the entries' nominal identities.

### Refinement: From Specification to Implementation

Refinement unfolds a high-level idea into lower-level design, implementation, and tests.
The refined entry keeps the meaning of the concept intact,
while making its consequences local and concrete.

A refinement chain is a path of increasing specificity.
It starts from a compressed concept and ends near repository text,
preserving the reason that a local decision exists.

If the programming language itself is expressive and clean enough such that
the logic of the design is the clearest when expressed in code,
then the final step of refinement may be a markdown code block.

A *refiner* relation is a metadata block field of the refined entry.
It points to the entry that it refines.

### Mirror Design with Witness

A *witness* relation is a metadata block field of the witnessed entry.
It points to code in repository that evidences the entry's claim.

When an entry describes behavior, representation, or invariant,
the witness is the concrete program text against which that claim can be checked.
A test may witness an entry when the test itself is the relevant code.

---

When project knowledge remains in comments, design notes, plan mode notes, or reviewer memory,
it is disconnected from both repository state and change propagation.
A code edit can invalidate a claim without identifying the dependent claims that must be re-examined.
A design edit can change an upstream commitment without updating the code witnesses.

---

## Layering

Sirno is defined above two smaller components:

- `eter`, which owns immutable snapshots, history, and the write transaction
  boundary
- `mosaika`, which owns repository alignment analysis and the discovery of witness

Sirno defines the knowledge semantics that use those components. It introduces
the design graph, its relations, and the write-acceptance rules that connect
graph state to repository state.

---

## Design Graph

The Sirno design graph is the intermediate representation between project-scale
design and repository-scale implementation.

The graph consists of entries together with explicit refinement, dependency, and
witness relations. Entry prose may also contain implicit associative links.

The graph is concept-driven in shape. Work begins from concept-bearing entries
and moves downward by refinement. Higher entries capture the named ideas that
compress intention. Lower entries unfold those ideas into specifications, work
items, and code-adjacent detail.

Refinement, dependency, and witness are load-bearing graph relations. They are
tracked explicitly because they carry operational consequences. Prose links are
navigational only.

---

## Core Concepts

### Entry

An entry is the primitive object in Sirno. An entry carries:

- a nominal identifier
- an optional human-readable name
- a concise description
- a full explanation

An entry states one claim about the codebase. The claim may describe an
invariant, a design decision, a representation choice, a concept, a
specification, a work item, or another isolated piece of understanding.

An entry owns explanatory prose. Other entry identifiers may appear in that
prose as links. These links are associative references. They do not create
propagation edges.

Concept-bearing entries are the preferred starting point for work. They carry
the highest compression of intention. A well-formed concept entry makes
lower-level specifications and implementation choices more local, more
predictable, and easier to review against their design purpose.

### Refinement

A refinement connects a more abstract entry to a more concrete one.

Refinement is the vertical structuring rule of the graph and the primary working
discipline in Sirno. It answers the question: how is this higher-level claim
elaborated into a lower-level design or implementation commitment.

Refinement does not imply reconsideration under change by itself. It organizes
the design from slogans and broad architecture down to code-adjacent detail.

Higher entries carry the named concepts and architectural claims that compress
intent. Lower entries unfold those concepts into specifications, work items, and
implementation details supported by code witnesses without severing the
connection to the original design meaning.

Work should therefore begin by locating the relevant higher-level entries and
following refinement downward. Local implementation is the end of this path, not
the start of it.

### Dependency

A dependency `X -> Y` states that `Y` must be re-examined when the content of
`X` changes.

Dependency direction is the direction of causal force. The source entry is the
claim being depended upon. The target entry is the claim whose validity depends
on the source.

A dependency may refer to an additional entry that explains what the dependency
means. That entry is descriptive metadata. The operational semantics of the
dependency are determined by the endpoints.

### Witness

A witness binds an entry to code that evidences the entry's claim.
The binding is stored as a Sirno witness specification interpreted through
`mosaika`.

A witness has three components:

- a source selection over code files
- one or more delimiter-based log transforms
- a Sirno interpretation of the resulting code regions

Sirno uses anchors and spans as witness machinery.

An anchor is a one-delimiter code region that marks the nominal presence of the
entry in program text.

A span is the code region selected for inspection, reflection, or actualization.

The selected span becomes a witness only when it is designated as evidence for
the entry's claim.

Witnesses operate over code in textual form. `mosaika` provides the alignment
analysis that discovers the witnessed code regions.

### Obligation

An obligation is the re-examination burden induced by a claim-bearing change.

A change is claim-bearing when it changes either:

- the text of an entry
- the dependency egress of an entry

Witness-specification changes, refinement changes, and lock-state changes are not
claim-bearing. They change code evidence, design organization, or
authority. They do not change downstream validity by themselves.

Obligations are derived from dependency under change. They are not a separate
persistent coupling concept in the graph.

### Lock

A lock is the authority boundary on claim-bearing writes to an entry.

A locked entry may be read, used during propagation, and checked against its
witnesses. Changing its claim-bearing fields requires external approval.

The approval path for a locked write carries the proposed graph write together
with an argument entry that explains the change. The rationale is therefore part
of the graph rather than transient review metadata.

Locks protect entries with wide consequences, such as architectural decisions,
global invariants, and externally promised guarantees.

---

## Storage and Write Model

Sirno is stored as an `eter` node schema. Every Sirno entry is an `eter` node.
The entry identifier is the `NodeId`. A durable Sirno state is an `eter`
snapshot identified by an `Eterator`.

The logical Sirno fields are:

- lifecycle
- entry name
- entry description
- entry explanation
- refinement egress
- dependency egress
- witness specifications
- lock state

The lifecycle field is the `eter` lifecycle field. Sirno uses it to determine
whether an entry exists at a snapshot.

Sirno chooses non-reuse of entry identifiers. Once an identifier has existed,
it remains reserved even after deletion. Nominal identity therefore persists
across the whole graph history.

Refinement and dependency egress are stored on the source entry. Reverse
adjacency is derived state.

Witness specifications are stored as typed Sirno data compatible with the
`mosaika` analysis model. Lock state is stored on the entry because authority is
part of graph state.

Sirno is used in two workflows. In actualization, the graph is authoritative
and repository code is rewritten to satisfy selected entries. In
reflection, the repository view is authoritative and observations from code
witnesses are written back into the graph.

In both workflows, work begins from graph exploration rather than from local
code inspection alone. The expected starting point is the relevant concept or
design entry and its refinement chain.

A write begins from one base `Eterator` and accumulates proposed Sirno field
writes relative to that snapshot. The write also carries the obligations induced
by those changes, the repository view used for validation, and any pending
locked-write approvals.

The durable write boundary is the `eter` transaction. The candidate Sirno state
is computed first, then validated against the repository view, then written as
new field rows. If the transaction succeeds, `eter` returns the new `Eterator`.

A write is accepted only when all induced obligations have been discharged, all
locked-entry changes have been approved, and the resulting witness
specifications pass the required repository validation.

Repository analysis occurs before the `eter` transaction. Repository
materialization, when actualization edits code, also occurs before the `eter`
transaction. The graph is written only after the repository view and the graph
view agree.

---

## Witness and Repository Alignment

The witness language is delimiter-based because it is interpreted through
`mosaika`. A witness identifies source-code files, declares delimiter-based log
transforms, and interprets the resulting regions as code evidence for entry
claims.

`mosaika` replacement actions belong to actualization tooling that rewrites
repository code to satisfy entries. Sirno uses the analysis side of `mosaika`
for witnesses.

Witness validation has three layers.

The first layer is specification validity. The source selection, delimiters, and
region interpretation must form a valid `mosaika` analysis specification.

The second layer is repository analysis. The `mosaika` analysis must resolve the
selected files and produce the required regions without ambiguity.

The third layer is Sirno interpretation. Anchors must bind to the owning entry.
Witnesses must remain evidentiary for the entry's claim. Required code regions
must be present.

Witnesses are evaluated relative to a repository view. In a repository-backed
deployment, that view is typically a checked-out tree plus any in-progress code
changes owned by the active task.

In reflection, witnessed code observations become proposed graph writes.
Sirno introduces no additional concept for that step beyond the write workflow
itself.

---

## Propagation

Propagation follows dependency edges in their declared direction.

When a write stages a claim-bearing change to entry `X`, Sirno computes the
dependency egress of `X` in the resulting graph. For each dependency `X -> Y`,
Sirno creates an obligation on `Y`.

An obligation is discharged in one of three ways.

Confirmation records that `Y` remains valid under the new upstream state.

Revision records new field writes for `Y`. If that revision is claim-bearing,
propagation continues from `Y`.

Approval records that a previously reviewed locked write to `Y` is accepted.
The approved writes are then applied and propagated in the same way as any
other revision.

Cycles are handled at the level of strongly connected components. Every entry in
the component must be re-examined against the same candidate state. The
component is discharged only when its entries reach a fixed point.

These propagation rules are part of write acceptance. An accepted write records
that all required reconsideration has been completed rather than deferred
silently.
