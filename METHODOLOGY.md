# Sirno Methodology

Sirno is a method for keeping design and implementation in conversation.
It assumes that a project has one readable narrative,
a store of compact named entries,
and a codebase that can witness those entries.

The method is not automatic understanding.
It is disciplined bookkeeping for people and agents who already intend to
understand.

Sirno should make the next design move easier to state,
the next implementation move easier to justify,
and the next review easier to ground in named project knowledge.

---

## What To Expect

A Sirno-assisted project should not feel like a pile of notes.
It should feel like a project whose design has handles.

The user can point at the long narrative and ask for it to be lowered into
entries.
The user can point at entries and ask for code to realize them.
The user can point at changed code and ask which entries should reflect the
change.
The user can ask for the long narrative to be raised from the current store.

Sirno supplies the shared structure for those requests.
It maintains entry ids, metadata, structural fields, generated footers,
storage conventions, and witness lookup.
It lets humans, LLMs, skills, CLIs, MCP tools, and editors speak about the same
objects without inventing a new map in every session.

Sirno does not decide whether a design is good.
It does not prove that code satisfies an entry.
It does not find design-code mismatch by itself.
It makes the relevant objects easier to name, inspect, connect, and revise.

---

## Keep One Monograph

Every Sirno project has one `mono` surface.
It is the configured long-form Markdown narrative,
often `DESIGN.md`.

The monograph is written for a reader who wants the project in one sitting.
It may omit local details in a large project,
but it should still describe the aim, intent, architecture,
and important design choices as a coherent document.

When a project has no Sirno store yet,
the monograph may be the best statement of intent.
When the store is established,
the monograph becomes the raised narrative view of the store.

Do not let the monograph become a dumping ground.
It should read as a narrative,
not as a directory of entries.

---

## Give Design A Nominal Form

The Sirno store is the `sirno` surface.
It contains Markdown entries.
Each entry is a nominal object:
its filename stem is its id,
and that id is the stable handle used by other entries and by witnesses.

An entry should be small enough to read locally.
It should state a concept, structural field, refinement, invariant,
implementation commitment, or narrative route with a name attached.

The required metadata fields are `name` and `description`.
Both are plain strings.

The structural fields are `category`, `clustee`, and `refiner`.
They are always lists of entry ids.

The `witness:` marker has no value.
It is either present in canonical form or absent.
When present, the entry id is the `mosaika` query key.

Operational structure comes from metadata.
Prose links may help readers and external tools,
but they do not carry Sirno structure.

---

## Lower Before Work Gets Local

Lowering moves from `mono` to `sirno`.
It takes broad narrative design and gives it compact named form.

Lower before implementation when the work would otherwise rely on memory,
chat context,
or a vague agreement about what the code is supposed to become.

Lowering does not require every part of the project to be decomposed at once.
It requires enough entries for the area being changed.

Good lowering preserves the intent of the monograph while making local work
addressable.
The result is not a task list.
It is a set of named objects that future work can refer to without retelling
the whole story.

---

## Realize From Named Objects

Realizing moves from `sirno` to `code`.
It is implementation guided by entries.

Before editing code,
read the entries that govern the work.
Follow their categories, clustees, refiners, and witnesses.
Then inspect the witnessed repository regions.

Implementation should be able to answer a simple question:
which entry explains why this code exists?

Sirno does not require every line of code to have a nearby entry.
It does require important design commitments to have a named place,
and it expects implementation work to use that place.

---

## Reflect After Code Changes

Reflecting moves from `code` to `sirno`.
It updates the store after implementation teaches the project something.

Reflect when code changes a representation,
narrows an invariant,
introduces a new boundary,
invalidates an old explanation,
or creates a clearer local design than the entries currently record.

Reflection should happen while the code change is still fresh.
Waiting turns design into archaeology.

The reflected entry does not need to narrate the whole edit.
It should record the durable design fact that future work needs.

---

## Raise For Whole-Project Reading

Raising moves from `sirno` to `mono`.
It composes the current store into a readable project narrative.

Raise when a reader needs the whole-project picture.
Raise when the monograph has fallen behind the store.
Raise before using the monograph as the next source of intent.

Raising is not concatenation.
The monograph should preserve a route through the project.
It should introduce terms once,
trust them afterward,
and omit local detail when that detail belongs in entries.

---

## Use Concepts For Compression

A concept is an entry that gives a name to compressed project knowledge.
It may cover design intention, algorithmic detail, local vocabulary,
behavioral specification, or test rationale.

Concepts are useful because they let people and agents refer to a bundle of
meaning without restating it.
They keep intent portable across the monograph, entries, and code.

Use a concept when several decisions share a reason.
Use a concept when a test property needs a name.
Use a concept when a local implementation choice should remain connected to a
larger idea.

The initialized `concept` entry is ordinary.
It is created by project setup,
not privileged by the system.

---

## Use Narratives For Cognitive Routes

A narrative is an entry that records a route through concepts.
The monograph is the primary narrative surface.

Materialized narratives may also live in the store as guides.
They can state prerequisites,
choose a base language,
and point the reader through the concepts in an intentional order.

Interactive narratives may be generated ephemerally by skills.
They adapt the route to the reader,
but the durable knowledge remains in entries.

The initialized `narrative` entry is ordinary.
It is a starting convention,
not a special case.

---

## Organize Without Hidden Semantics

Use `category` to classify an entry by other entries.
Use the `meta` category for entries that define categories.
There is no separate `meta` field.

Use `clustee` when entries share a subject,
local vocabulary,
or design neighborhood.
The named clique closure is itself an ordinary entry.
It gives the group a place to be named and explained.

Use `refiner` when a more specific entry elaborates one or more broader
entries.
Refinement is how broad design becomes local design,
implementation detail,
or testable behavior.

These fields should make navigation and responsibility clearer.
They should not smuggle in unstated rules.

---

## Witness The Repository

Use `witness:` when the repository contains evidence for an entry.

The witness may be source code, tests, configuration, generated files, assets,
or any artifact that `mosaika` can mark and query.
Sirno queries witnesses by entry id.

The entry body may explain how to interpret the evidence.
That prose is helpful,
but the structural convention is the marker and the id.

Witnesses make review concrete.
They let a reader move from a named design claim to the repository artifacts
that should be inspected.

---

## Let Sirno Maintain The Edges Of The Page

Sirno may maintain generated footers at the bottom of entries.
The footer can use Markdown links or Obsidian-style links,
depending on project configuration.

The generated region is bounded by sentinels.
The sentinels say that Sirno owns the region.
Humans and other tools should leave it untouched.

Generated footers exist for navigation and interoperability.
They reflect Sirno-managed structure for tools that prefer links.

---

## Check At Review Boundaries

During editing,
Sirno may warn about dangling `category`, `clustee`, and `refiner` ids.
At an explicit check boundary,
those dangling ids are errors.

Witness validity is checked only during explicit checks.
This keeps editing light and review strict.

Checks are structural.
They confirm that ids, metadata, footers, and witness lookup conventions are in
order.
They do not replace human or agent judgment about whether the design and code
mean the same thing.

---

## Treat Planning As A Use, Not A Primitive

Sirno does not define planning as a core field or phase.
It gives people and agents a stable structure that can support planning.

A Sirno-provided skill may represent a worklist as entries.
That worklist can use categories, refiners, clustees, and witnesses like any
other entry set.

The planning artifact is a use of Sirno.
It is not part of Sirno's core ontology.

---

## The Habit

Name the thing.
Write the entry.
Classify it only when classification helps.
Cluster it only when the shared subject deserves a name.
Refine it when broad design needs local form.
Witness it when the repository contains its evidence.

Lower before local work loses its source.
Realize from named objects.
Reflect while the code change is still fresh.
Raise when the project needs to be read as one document.

Sirno keeps the structure ready.
People and agents keep the meaning alive.
