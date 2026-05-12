---
name: sirno-lowering
description: >-
  Lower a Sirno monograph or design document into a Sirno entry store. Use when Codex transforms
  DESIGN.md or another mono surface into compact Markdown entries, chooses entry ids and structural
  metadata, expands lowered entries, or validates generated Sirno links.
---

# Sirno Lowering

## Purpose

Use this skill when moving design knowledge from `mono` to `sirno`.
Lowering gives a long-form narrative compact nominal form.
It preserves the design route while creating entries future work can cite, query, realize, and reflect.

Apply repository instructions first.
When the work also edits `README.md`, `DESIGN.md`, or `METHODOLOGY.md`,
use the repository documentation-writing skills for prose style and document roles.

## Core Judgment

Lowering is not a heading split.
Read the monograph as a whole,
then name the durable objects that make future work easier to address.

Look for:

- concepts that compress repeated project knowledge
- categories that define project vocabulary
- entries that explain `category`, `clustee`, `refiner`, and `witness:`
- direction entries that explain movement between surfaces
- interface, storage, metadata, and checking commitments
- witnessable claims that should connect to repository artifacts
- narrative routes that help a reader traverse concepts

Prefer entries that a future implementer or reviewer can point at.
Avoid entries that only restate a paragraph without creating a useful handle.

## Authority

Before lowering, decide which surface currently carries authority.
If the store is already established and maintained,
treat it as the structured design source.
If the store is absent or skeletal,
use the configured monograph as the source of intended design unless the user says code is authoritative.

Preserve stable facts from the current project model:

- central definition and scope
- configured `mono` and `store` paths
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

## Structural Field Heuristics

Use `category` for kind.
An entry categorized by `concept` should define a compressed idea.
An entry categorized by `narrative` should record a route through concepts.
An entry categorized by `meta` should define project vocabulary.

Use `clustee` for neighborhood.
The target is a clique closure entry that gives a shared subject or design region a front door.
Use it when entries of different kinds should be visited together.

Use `refiner` for specificity.
The more specific entry points to the broader entry it makes local or concrete.
Use it to preserve the reason behind implementation detail, invariants, interfaces, or tests.

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

Avoid turning the store into:

- a glossary with labels but no design pressure
- a changelog that narrates edits instead of durable facts
- a task list that loses the concept behind the work
- a duplicate monograph split across files

Entries should be more local than the monograph,
but more durable than a plan item.

## Workflow

1. Read repository instructions, `Sirno.toml`, the monograph, and the existing store.
2. Inspect the current Sirno CLI before assuming which commands exist.
3. Map candidate entries before editing:
   ids, names, descriptions, categories, clustees, refiners, and witness markers.
4. Create missing entries through Sirno's current entry-creation surface when available.
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

When asked to commit lowering work,
stage only the configured store, the config change that points to it,
and directly related documentation.
Leave unrelated code or generated editor state alone.

Use the repository commit convention.
For documentation-only lowering, `docs: lower design into sirno store` is an appropriate shape.
