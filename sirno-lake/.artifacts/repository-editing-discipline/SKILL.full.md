---
name: sirno-editor
description: >-
  Edit a Sirno-managed repository by reading its configured lake before source, docs, config,
  witness, or skill edits, then syncing entries and validating the result.
---

# Sirno Editor

## Purpose

Use this skill for any edit to a Sirno-managed repository.
Start in the active project's configured lake.
Read the entries that explain the work,
update those entries when the design changes,
then edit repository material under that named design.

Repository material covers source, tests, generated artifacts, configuration,
README files, design documents outside the configured lake, witness blocks, and skill packages.
Any important local commitment should trace back to an entry in the active project.

This full skill text is served as `sirno://skills/sirno-editor`.
Apply it to the project currently bound through Sirno MCP.

## Project Binding

Bind the MCP server to the repository before calling project tools.
Call `sirno_cwd` with the repository root when the server may not already be there.
Project tools resolve `Sirno.toml` from the server current working directory on each call.
Call `sirno_cwd` again before switching projects in the same server process.

## Workflow

1. Map the project.
   Read repository instructions and `Sirno.toml`.
   If `Sirno.toml` is missing, report that the repository is not Sirno-managed
   and prompt the user to start with `sirno init`.
   Call `sirno_status` to surface the lake path, tide blockers,
   and any pending review entries.
   Query the active lake with `sirno_entry_query`,
   then read the few candidate entries that govern the request.
   Follow configured link relations such as `category`, `belongs`, `prerequisite`, and `refines`.
   Use `sirno_entry_witness` to inspect existing evidence before touching repository material.

2. Choose the design handle.
   If an existing entry names the right commitment, work under that entry.
   If the change introduces a new boundary, invariant, representation, behavior, or policy,
   create or revise a compact entry before the repository change settles.
   Apply semantic locality to every entry creation or revision:
   the body should state the local meaning in place,
   and broad entries should not inventory their current implementations.
   Use `sirno_entry_new` for a new entry,
   `sirno_entry_rename` to change its id,
   and `sirno_entry_artifact_*` for artifacts owned by an entry.
   Apply link relations only when they improve navigation, review, or accountability.
   Leave generated footer regions untouched.

3. Edit the repository.
   Make the source, test, document, artifact, config, or skill change from the updated entry.
   Keep the implementation narrow and aligned with the entry claim.
   When evidence exists, add or refine precise witness blocks around the smallest stable region
   that supports the entry.
   Do not add placeholder witnesses.
   If evidence supports a related but different claim, create the exact entry for that claim.

4. Keep configuration aligned.
   Prefer MCP tools such as `sirno_lake_move`, `sirno_anchor_status`, `sirno_anchor_check`,
   `sirno_anchor_update`, and `sirno_entry_artifact_*`
   for routine lake movement, Anchor checks, Anchor updates, and artifact moves.
   Use manual `Sirno.toml` edits only for schema work or comment maintenance the MCP tools
   cannot express,
   then run deterministic config repair when available.
   Preserve path rules from `Sirno.toml` and link relation meaning from the active project.
   Add `[repo].members` paths only when they are intended witness surfaces.

5. Sync public documentation.
   Treat long-form docs outside the configured lake as repository material.
   Keep durable design claims in entries,
   then update public prose from those entries.
   Use the repository's own documentation-writing method when one exists;
   otherwise fall back to `sirno://skills/design-doc-writer`.
   Use paragraphs, bullets, numbered steps, tables, or simple diagrams
   according to what makes the design easiest for a human co-worker to scan and review.

## Validation

Run `sirno_lake_render` after lake metadata changes.
Run `sirno_lake_check` in edit mode and review mode before treating the work as complete.
Run `sirno_entry_witness` against any entry whose evidence changed.
Run the formatter, tests, and checks that fit the active repository.

If `sirno_status` reports an open tide,
walk workitems with `sirno_tide_status`,
then use `sirno_tide_resolve` or `sirno_tide_unresolve` rather than ignoring the blocker.
If the current checkout is frozen or an entry is immutable,
use `sirno_entry_melt`
instead of forcing a write.

If a check is blocked, report the blocker and still validate entry parsing,
metadata references, and witness output as far as the tools allow.

Stage only the changed entries and artifacts,
the repository files that actualize them,
and directly related configuration or documentation.
