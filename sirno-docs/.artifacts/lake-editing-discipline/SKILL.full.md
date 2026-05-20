---
name: sirno-editor
description: >-
  Edit a Sirno Lake. Use for creating, revising, or reorganizing compact Markdown entries,
  internalizing durable repository knowledge into the lake, choosing entry ids and structural
  metadata, expanding entries, or validating generated Sirno links.
---

# Sirno Editor

## Purpose

Use this skill when editing Sirno Lake entries
or internalizing durable repository knowledge into the lake.
Editing keeps the lake precise, structured, and useful for future work.
Internalization gives repository material compact nominal form.
It preserves durable design knowledge while creating or revising entries
future work can cite, query, actualize, internalize, and witness.
This full skill text is served as the MCP resource `sirno://skills/sirno-editor`.
It follows the `lake-editing-discipline` lake entry.

Apply repository instructions first.
When the work also edits `README.md`, `DESIGN.md`, or `METHODOLOGY.md`,
use the repository documentation-writing skills for prose style and document roles.

## MCP Project Resolution

When using Sirno through MCP, call `sirno_cwd` with the repository root before project tools
if the server started without `--config`.
Project tools resolve `Sirno.toml` on every project tool call from the current server cwd.
Call `sirno_cwd` again before switching projects in the same server process.

## Core Judgment

Sirno Lake editing is not a heading split or metadata shuffle.
When internalizing from repository material,
read the relevant material and its surrounding context,
then name the durable objects that make future work easier to address.
When revising existing entries,
preserve stable ids and improve the structure that future work will cite.

Look for:

- concepts that compress repeated project knowledge
- categories that define project vocabulary
- entries that explain recommended structural fields and witness lookup
- transform entries that explain movement between forms
- interface, storage, metadata, and checking commitments
- witnessable claims that should connect to repository artifacts
- narrative routes that help a reader traverse concepts

Prefer entries that a future implementer or reviewer can point at.
Avoid entries that only restate a paragraph without creating a useful handle.

## Authority

Before editing, decide which form currently carries authority.
If the lake is already established and maintained,
treat it as the canonical design source.
If the lake is absent or skeletal,
use repository material as input to internalize,
and ask the user when repository materials disagree.

Preserve stable facts from the current project model:

- central definition and scope
- configured `lake` path and repository material surface
- entry id rules and metadata schema
- configured structural field and witness lookup meanings
- generated footer ownership
- witness lookup conventions
- future-work items that are intentionally reserved

Translate stale language into current terms rather than preserving it literally.
Do not invent tools, guarantees, semantic understanding, or automatic validation that Sirno does not provide.

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

This repository recommends the following structural field model.
Check `Sirno.toml` before relying on a field in commands or generated links.

Use `category` for kind.
An entry categorized by `concept` should define a compressed idea.
An entry categorized by `narrative` should record a route, story, or motivation through project ideas.
An entry categorized by `meta` should define the project's principles, vocabulary, or documentation method.
An entry categorized by `category` may itself be used as a category target.
The local documentation method requires that category targets are categorized by `category`.

Use `belongs` for review locality.
A `belongs` target is an entry that gives a shared subject or design region a front door.
It says that entries should be visited together because they live in the same working neighborhood.
The relation is horizontal.
It supports scanning, review, accountability, and local navigation across entries of different kinds.

Use `refines` for semantic narrowing.
A `refines` target is the broader entry that the current entry makes more specific.
It says that the current entry is a local, concrete, or testable version of another design claim.
The relation is vertical.
It preserves why an implementation detail, invariant, interface, route, or test belongs under a broader idea.

Prefer choosing either `belongs` or `refines` for a new entry.
They are suggested to be mutually exclusive because they answer different questions.
`belongs` answers "which review neighborhood contains this entry?"
`refines` answers "which broader claim does this entry specialize?"
Using both can blur locality and specificity,
so add both only when the entry truly sits in a review neighborhood
and also concretizes a broader design claim that should be followed separately.

When choosing `belongs`,
prefer the smallest set of targets that improves navigation, review, or accountability.
An entry may name several `belongs` targets only when each target is a real review perspective.
Keep split entries under the same `belongs` target when a small design change should be checked inside that unit.
Create a new `belongs` target only when there is a real new review boundary.

When choosing `refines`,
prefer the nearest broader entry that explains the current entry's design pressure.
Do not use `refines` to say that two entries are merely related or commonly edited together.
Create a more specific entry when a paragraph, code region, test, or policy needs a stable handle.

The entry id is the witness query key.
Discover evidence with `sirno_entry_witness`.
The body should briefly explain what the repository evidence is expected to demonstrate
when that meaning is not obvious from the entry claim.

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
Use section headings only when they frame the material that follows.
Do not leave a heading empty by placing another heading, diagram, or list directly under it.

Avoid turning the lake into:

- a glossary with labels but no design pressure
- a changelog that narrates edits instead of durable facts
- a task list that loses the concept behind the work
- a duplicate repository document split across files

Entries should be more canonical than repository material,
but more local than a whole-project dump.

## Workflow

1. Read repository instructions, `Sirno.toml`, relevant repository material, and the existing lake.
   Use the configured lake as the routine edit target.
   `sirno-docs-zh/` stores the split Chinese translation snapshot.
   Leave that directory unchanged during lake maintenance and design sync.
2. Inspect the current Sirno MCP tools before assuming which operations exist.
3. Map candidate entries before editing:
   id, name, desc, structural fields, and witness status.
4. Create missing entries through `sirno_entry_new` when available.
5. Expand or revise bodies with direct, reader-friendly prose.
6. When editing design documents or design entries,
   use the repository's own design-document skill or documented manner first.
   If none exists, default to the discipline in `sirno://skills/design-doc-writer`,
   documented by `design-doc-writer-skill`:
   read the whole design route, order concepts by dependency and scope,
   write declarative, dry, precise prose, merge avoidable overlap,
   and keep one idea per paragraph.
7. Leave generated footer regions untouched.
8. Run `sirno_lake_render` after metadata stabilizes.
9. Run `sirno_lake_check` with `mode: "edit"`.
10. Run `sirno_lake_check` with `mode: "review"` before treating the edit as complete.
11. Run query tools to verify the lake parses and references resolve.

Use the configured lake path.
Do not hard-code `docs/` when `Sirno.toml` names a different lake.
Use `sirno-config-writer` when the design change requires `Sirno.toml` edits.
Lake editing may change entries that describe config,
but config-writing rules live in `sirno://skills/sirno-config-writer`.

## Document Search

Use `sirno_entry_query` to map concepts, structural neighborhoods, and candidate entry ids.
Read the `desc` field before deciding which entries to edit.

Use `sirno_entry_rg` to search literal text in Sirno documents:
phrases, command names, examples, stale wording, headings, or entry ids used in prose.
Plain `sirno_entry_rg` searches authored metadata and prose.
It ignores generated footer regions by default.
Set `with_generated_footer: true` only when generated links are the search target.

Useful document-search tool inputs:

```json
{"terms": ["TERMS"], "columns": ["id", "desc"]}
{"has": "FIELD=ENTRY_ID", "columns": ["id", "path", "desc"]}
{"args": ["TEXT"]}
{"args": ["-n", "TEXT"]}
{"args": ["-C", "2", "TEXT"]}
{"args": ["--files"]}
```

After finding literal matches,
read the matched entries before editing.
Do not rewrite from isolated match lines alone.

## Validation

Prefer these MCP checks:

```text
sirno_entry_query
sirno_entry_rg
sirno_lake_check mode=edit
sirno_lake_render
sirno_lake_check mode=review
sirno_status
```

If review-mode checks fail because local editor/tool directories are inside the lake,
preserve those files unless the user asks to remove them.
Report the blocker and still validate entry parsing and metadata references as far as possible.

If the entry is frozen or a checkout is immutable,
use the configured frost workflow before editing instead of forcing a write.
If a tool named by older guidance is missing,
inspect the current MCP tool list and use the closest current tool only when its behavior is clear.
If authored metadata, references, or generated-footer freshness fail,
fix the lake before treating the edit as complete.

## Git Hygiene

When asked to commit Sirno Lake editing work,
stage only the configured lake, the config change that points to it,
and directly related documentation.
Leave unrelated code or generated editor state alone.

Use the repository commit convention.
For documentation-only lake editing, `docs: revise sirno lake entries` is an appropriate shape.
