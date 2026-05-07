# Sirno Methodology

Sirno is a methodology for design-led software development with durable
alignment between design, planning, and code.

The method starts from a comprehensive `DESIGN.md`. It then elaborates that
document into a graph of smaller design entries, grounds those entries to the
codebase, and enforces re-examination when a local change affects other parts of
the design. The result is a persistent blueprint that remains useful after the
initial code generation step.

The method uses three layers:

- `DESIGN.md`, which states the whole design in one human-readable document
- the Sirno design graph, which refines that design into structured entries and
  relations
- the codebase, which realizes the refined design in executable artifacts

The design graph is the intermediate representation between the broad design
document and the concrete codebase.

---

## Motivation

Project design usually begins in a form that is broad, coherent, and readable.
As implementation proceeds, that design is fragmented across code, comments,
issues, review threads, and planning sessions. The original design intent is
still present, but it is no longer explicit, localizable, or propagatable.

One-time code generation does not solve this problem. It can produce an initial
codebase from a polished design document, but the resulting codebase is often a
dead end from the perspective of design alignment. The generated code has no
durable mechanism for recording which design commitments it realizes, which
higher-level concepts it refines, or which neighboring decisions must be
reconsidered when it changes.

Sirno treats this gap as an intermediate-representation problem. `DESIGN.md`
contains the whole design at document scale. The codebase contains the realized
implementation at artifact scale. Between them sits a graph of entries that is
small enough to manipulate locally and structured enough to preserve global
design intent.

---

## Starting Point

The starting point of the method is a project-level `DESIGN.md`.

`DESIGN.md` is written first and polished until it reaches a stable statement of
the project's major ideas, constraints, architecture, and agenda. It is not
required to contain every implementation detail. It must contain the design at a
level where refinement can begin without inventing the project direction during
coding.

An initial codebase may be generated from this design state. That generation
step is a bootstrap step, not the whole method. The generated codebase becomes
maintainable only when it is brought under the design graph and its entries.

Sirno begins when the project is ready to move from whole-document design to
structured refinement.

---

## Core Artifacts

### DESIGN.md

`DESIGN.md` is the top-level design statement of the project. It is written for
whole-project readability. Its job is to define the system, its major decisions,
and its intended shape.

`DESIGN.md` is authoritative at document scale. It says what the project is.

### Entry

An `Entry` is the text-oriented node of the design graph.

An `Entry` is a Markdown file with metadata in its header and explanatory prose
in its body. The body carries the actual design content. The header carries the
structured data needed for graph behavior, refinement discipline, grounding, and
obligation propagation.

An `Entry` is smaller and tighter than `DESIGN.md`. It should state one claim,
one concept, one refinement step, one implementation commitment, or one work
item with a stable design meaning.

### Design Graph

The design graph is the persistent intermediate representation between
`DESIGN.md` and the codebase.

The graph organizes entries into a structure that supports:

- refinement from broad design to local implementation detail
- grounding from design intent to code regions
- propagation from local changes to affected entries
- persistent planning and review

The graph is the durable working state of the project blueprint.

### Codebase

The codebase is the implementation surface realized from the design graph.

It is not treated as independent from the design. Repository artifacts are
grounded to entries so that implementation remains attributable to design
intent, and design remains checkable against implementation.

---

## Refinement

Refinement is the primary construction rule of the method.

The method starts from concise and overarching ideas and progressively refines
them into more detailed entries. A higher-level entry captures an intention,
constraint, or architectural commitment. A lower-level entry captures a more
detailed realization of that same commitment.

A refinement edge therefore connects design scale to implementation scale
without collapsing the two.

The highest entries may be slogans, agendas, major architectural principles, or
concise design claims. Intermediate entries may define subsystems, protocols,
data models, invariants, workflows, and interfaces. Lower entries may define
algorithms, pseudocode, module-local design, or work items close to actual code.
The lowest entries may correspond almost directly to implementation.

Refinement is gradual by design. The method does not require the whole graph to
be elaborated uniformly before implementation begins. It requires that any area
being implemented has enough refined design context to support aligned work.

Refinement and obligation are distinct edge types.

Refinement is vertical. It answers the question: how is this higher-level claim
elaborated into a more concrete one.

Obligation is lateral or upward. It answers the question: what else must be
reconsidered when this claim changes.

---

## Concepts Before Implementation

Sirno is concept-driven.

Between specification and implementation, the method refines through concepts.
Concepts capture intention at a scale that is small enough to manipulate and
large enough to compress detail. A good concept says what several local
implementation choices are for, not merely what they are.

This makes the design graph closer to a project wiki than to a task list, but it
is a wiki with graph semantics and operational consequences.

Concept-driven refinement serves three purposes:

- it preserves story-level understanding while details accumulate
- it gives local implementation decisions a named design home
- it improves compression for both human review and model reasoning

An implementation entry without a concept above it is suspect. A concept entry
without any plausible route to implementation is incomplete. Refinement keeps
both sides accountable.

---

## Persistent Planning

Sirno treats planning as durable design state rather than disposable chat
output.

Ordinary planning sessions with an LLM are often one-time artifacts. They may be
useful in the moment, but they are rarely integrated into the project's durable
knowledge. Sirno replaces this with persistent planning in the design graph.

A reviewable and reproducible worklist is represented as entries and relations
inside the graph. Planning therefore becomes part of the design blueprint
itself. A work item is not only a todo; it is a refinement step that is related
to design intent, neighboring work, and implementation groundings.

The graph therefore acts as a persistent plan mode:

- previous planning work is retained
- planning results are organized rather than merely archived
- future work begins from the existing design state instead of from an empty
  session

The method is compatible with interactive planning by an LLM. The expected
integration is that the model explores the existing graph first, proposes new or
revised worklist entries, and leaves behind reviewable structured state rather
than ephemeral prose.

---

## Obligations

Obligations are the anti-local-maximum rule of the method.

A local change to one entry is not allowed to remain purely local when other
entries depend on it. A design decision may be easy to optimize in isolation and
wrong in project context. Obligations force reconsideration of the affected
parties of interest.

When an entry changes, the graph identifies the entries that must be
reconsidered in response. Those entries receive obligations. The change is not
complete until those obligations are discharged.

Discharging an obligation may confirm that the dependent entry remains valid, or
it may require a further refinement or implementation change. In either case the
graph records that the consequence of the upstream change was examined rather
than ignored.

The obligation protocol has three steps:

1. Surface the obliged entries and, when useful, a bounded transitive closure of
   their own obligations.
2. Assess whether the upstream change invalidates, weakens, or leaves intact
   the obliged entry's claims.
3. Resolve by confirming validity, propagating a corresponding change, or
   recording an explicit deferral with rationale.

This mechanism is the methodological core of alignment maintenance. It prevents
the design graph from degrading into a disconnected collection of notes.

---

## Grounding and Alignment

Grounding is the connection between an entry and repository artifacts.

An entry may be attached to links or pointers to regions in the codebase that
realize, witness, or otherwise relate to the entry's content. These links make
design claims checkable against implementation and make code exploration start
from design intent rather than from raw syntax alone.

The machinery for grounding is `mosaika`.

Grounding supports alignment in both directions:

- from design to code, by locating the repository artifacts that realize an
  entry
- from code to design, by locating the entry or entries whose claims explain a
  repository region

Grounding also makes design review concrete. A high-level claim can be followed
through its refinements and then through its grounded implementation regions.

---

## Exploration Before Editing

No edit to the codebase should be permitted until sufficient exploration of the
design graph.

This rule prevents implementation from outrunning design context. Before any
code is changed, the relevant entries, refinements, obligations, and grounded
regions must be explored. The exploration step establishes:

- what claim is being implemented or changed
- what higher-level decisions govern that claim
- what neighboring entries may be affected
- what repository regions are already grounded to the relevant design

This is a methodological guardrail, not a convenience feature. It is what keeps
coding aligned with the project blueprint instead of reverting to local
trial-and-error.

The persistence machinery for this graph-first discipline is `eter`. The design
graph is durable, reviewable state that survives across sessions.

---

## Entry Structure

An entry contains two kinds of information.

The body contains the design text: the concept, refinement, work item,
implementation note, or review argument that the entry exists to state.

The header contains the structured metadata needed for graph behavior. This
includes explicit relations, grounding declarations, refinement status, and any
other machine-checked data that must not be inferred from prose alone.

Edges in the graph may arise in two ways:

- implicitly, from references in the Markdown body text
- explicitly, from metadata tracked in the header

This hybrid structure preserves readable prose while still giving the graph a
reliable operational substrate.

Implicit edges are weaker than explicit graph relations. They are suitable for
association and navigation. Refinement, obligation, and grounding should remain
explicit in metadata because they carry operational consequences.

---

## Method in Practice

The method proceeds in the following order.

1. Write and polish `DESIGN.md` until the project direction is stable enough to
   support refinement.
2. Generate the initial codebase if desired, treating that generation as a
   bootstrap step rather than as the completed design process.
3. Elaborate `DESIGN.md` into entries that isolate major claims, concepts,
   subsystems, and development agenda.
4. Refine high-level entries into lower-level entries until the area being
   worked on has enough context to support implementation.
5. Record planning output as entries in the graph rather than as disposable
   session text.
6. Explore the relevant design graph before editing the codebase.
7. Ground entries to repository artifacts and use those groundings during code
   work and review.
8. When an entry changes, discharge the obligations created for affected
   entries.
9. Continue refinement and implementation together so that the graph remains the
   live project blueprint rather than a frozen design artifact.

The output of the method is not only a codebase. It is a codebase together with
an aligned, persistent, and refinable design blueprint.

---

## Design Rationale

The design graph is a graph rather than a tree because design couplings are not
purely hierarchical. Refinement is hierarchical, but obligations and grounding
cross those boundaries.

The method uses obligations rather than a generic notion of dependency because
the intended meaning is reconsideration under change, not mere existence or
build-order reliance.

Grounding is external to the code rather than embedded as comments because the
relation must be traversable, reviewable, and independently checkable.

Concepts sit between slogans and code because they carry the highest
compression of intention. They are the layer most capable of preserving design
meaning while remaining generative for further refinement.

---

## Intended Result

The intended result of Sirno is a software project whose design remains durable
after implementation begins.

The project keeps:

- a readable whole-project design document
- a persistent graph of refined design entries
- grounded links between design and code
- obligations that force reconsideration of affected design when local changes
  occur
- planning state that survives and remains organized across sessions

This is the methodological meaning of a semantic intermediate representation of
nominal obligations. The intermediate representation is the design graph. The
nominal units are entries. The obligations preserve non-local alignment.
