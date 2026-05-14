---
name: sirno-editor
description: >-
  Edit a Sirno entry store. Use when Codex creates, revises, or reorganizes compact Markdown
  entries, moves design knowledge from DESIGN.md or another mono form into the store, chooses entry
  ids and structural metadata, expands entries, or validates generated Sirno links.
---

# Sirno Editor

## Purpose

Use this skill when editing `sirno` entries
or moving design knowledge from `mono` to `sirno`.
Editing keeps the store precise, structured, and useful for future work.
Lowering gives a long-form narrative compact nominal form.
It preserves the design route while creating or revising entries
future work can cite, query, realize, and reflect.

Apply repository instructions first.
When the work also edits `README.md`, `DESIGN.md`, or `METHODOLOGY.md`,
use the repository documentation-writing skills for prose style and document roles.

## Core Judgment

Sirno store editing is not a heading split or metadata shuffle.
When lowering from a monograph,
read the monograph as a whole,
then name the durable objects that make future work easier to address.
When revising existing entries,
preserve stable ids and improve the structure that future work will cite.

Look for:

- concepts that compress repeated project knowledge
- categories that define project vocabulary
- entries that explain `category`, `clustee`, `refiner`, and `witness:`
- transform entries that explain movement between forms
- interface, storage, metadata, and checking commitments
- witnessable claims that should connect to repository artifacts
- narrative routes that help a reader traverse concepts

Prefer entries that a future implementer or reviewer can point at.
Avoid entries that only restate a paragraph without creating a useful handle.

## Authority

Before editing, decide which form currently carries authority.
If the store is already established and maintained,
treat it as the structured design source.
If the store is absent or skeletal,
use the configured monograph as the source of intended design unless the user says code is authoritative.

Preserve stable facts from the current project model:

- central definition and scope
- configured `store` path and optional `mono` path
- entry id rules and metadata schema
- `category`, `clustee`, `refiner`, and `witness:` meanings
- generated footer ownership
- witness lookup conventions
- future-work items that are intentionally reserved

Translate stale language into current terms rather than preserving it literally.
Do not invent commands, guarantees, semantic understanding, or automatic validation that Sirno does not provide.

## Entry Design

Each entry should be small enough to read locally,
but substantial enough to stand on its own from a query result.
For foundational design entries, aim for reader-friendly prose of roughly a few paragraphs.
Shorter entries are fine when the concept is genuinely narrow.

Each entry should answer:

1. What does this name mean?
2. Why does it deserve a stable entry id?
3. How does it fit the project model?
4. Which structural metadata should carry its shape?
5. What should future implementation or review remember?

Use exact metadata for structure and prose for explanation.
Do not ask tools to infer structure from body text.

Use lowercase ASCII kebab-case ids.
Keep existing ids stable unless the user explicitly asks for a rename.
Use human-readable names and concise descriptions.

## Structural Field Model

Use `category` for kind.
An entry categorized by `concept` should define a compressed idea.
An entry categorized by `narrative` should record a route, story, or motivation through project ideas.
An entry categorized by `meta` should define project vocabulary.

Use `clustee` for review locality.
A clustee target is a clique closure entry that gives a shared subject or design region a front door.
It says that entries should be visited together because they live in the same working neighborhood.
The relation is horizontal.
It supports scanning, review, accountability, and local navigation across entries of different kinds.

Use `refiner` for semantic narrowing.
A refiner target is the broader entry that the current entry makes more specific.
It says that the current entry is a local, concrete, or testable version of another design claim.
The relation is vertical.
It preserves why an implementation detail, invariant, interface, route, or test belongs under a broader idea.

Prefer choosing either `clustee` or `refiner` for a new entry.
They are suggested to be mutually exclusive because they answer different questions.
`clustee` answers "which review neighborhood contains this entry?"
`refiner` answers "which broader claim does this entry specialize?"
Using both can blur locality and specificity,
so add both only when the entry truly sits in a review neighborhood
and also concretizes a broader design claim that should be followed separately.

When choosing `clustee`,
prefer the smallest set of clustees that improves navigation, review, or accountability.
An entry may belong to several clustees only when each clustee is a real review perspective.
Keep split entries in the same clustee when a small design change should be checked inside that unit.
Create a new clustee only when there is a real new review boundary.

When choosing `refiner`,
prefer the nearest broader entry that explains the current entry's design pressure.
Do not use `refiner` to say that two entries are merely related or commonly edited together.
Create a more specific entry when a paragraph, code region, test, or policy needs a stable handle.

Use `witness:` only when repository evidence exists or the task explicitly asks to declare it.
The entry id is the witness query key.
The body may explain how to interpret evidence,
but the structural convention is the marker.

When a structural field feels merely decorative,
leave it out.
Structural fields should improve navigation, review, or accountability.

## Prose Style

Write entries as durable design prose.
Define the positive rule first.
Use definition by negation only when it prevents a likely confusion.

Keep paragraphs focused.
One paragraph should introduce one idea, state its consequence, and stop.
Prefer short sentences and natural line breaks under the repository line budget.
Break Markdown prose at natural punctuation boundaries or conjunctions; otherwise don't make linebreaks.

Avoid turning the store into:

- a glossary with labels but no design pressure
- a changelog that narrates edits instead of durable facts
- a task list that loses the concept behind the work
- a duplicate monograph split across files

Entries should be more local than the monograph,
but more durable than a plan item.

## Workflow

1. Read repository instructions, `Sirno.toml`, the configured monograph when present, and the existing store.
2. Inspect the current Sirno CLI before assuming which commands exist.
3. Map candidate entries before editing:
   ids, names, descriptions, categories, clustees, refiners, and witness markers.
4. Create missing entries through Sirno's current entry-creation command when available.
5. Expand or revise bodies with direct, reader-friendly prose.
6. Leave generated footer regions untouched.
7. Run Sirno's generated-link command after metadata stabilizes.
8. Run structural checks and query commands to verify the store parses and references resolve.

Use the configured store path.
Do not hard-code `docs/` when `Sirno.toml` names a different store.

## Validation

Prefer these checks when the CLI provides them:

```text
sirno query --format id
sirno check --mode edit
sirno gen-link
sirno check
sirno status
```

Use `cargo run -- ...` or `target/debug/sirno ...` according to the repository state.

If review-mode checks fail because local editor/tool directories are inside the store,
preserve those files unless the user asks to remove them.
Report the blocker and still validate entry parsing and metadata references as far as possible.

## Git Hygiene

When asked to commit Sirno store editing work,
stage only the configured store, the config change that points to it,
and directly related documentation.
Leave unrelated code or generated editor state alone.

Use the repository commit convention.
For documentation-only store editing, `docs: revise sirno store entries` is an appropriate shape.
