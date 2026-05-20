---
name: sirno-maintainer
description: >-
  Maintain a Sirno-managed repository by reading its configured lake before source, docs, config,
  witness, or skill edits, then syncing entries and validating the result.
---

# Sirno Maintainer

## Purpose

Use this skill for edits to a repository that uses Sirno.
The first step is always the active project's configured lake.
Read the entries that explain the work,
update those entries when the design changes,
then edit repository material under that named design.

Repository material includes source, tests, generated artifacts, configuration,
README files, design documents outside the configured lake, witness blocks, and skill packages.
Important local commitments should be traceable to entries in the active project.

This full skill text is served as `sirno://skills/sirno-maintainer`.
Apply it to the project currently bound through Sirno MCP.

## Project Binding

Before calling project tools through MCP,
bind the server to the repository you are working in.
Call `sirno_cwd` with the repository root when the server may not already be there.
Project tools resolve `Sirno.toml` from the server current working directory.
Call `sirno_cwd` again before switching projects.

## Workflow

1. Find the project map.
   Read repository instructions and `Sirno.toml`.
   If `Sirno.toml` is missing, report that the repository is not currently Sirno-managed,
   then prompt the user to start with `sirno init`.
   Use the configured lake path from that file.
   Query the active lake with `sirno_entry_query`,
   then read the few candidate entries that govern the request.
   Follow configured structural fields such as `category`, `belongs`, `prerequisite`, and `refines`.
   Use `sirno_entry_witness` to inspect existing evidence before editing repository material.

2. Choose the design handle.
   If an existing entry names the right commitment, work under that entry.
   If the change introduces a boundary, invariant, representation, behavior, or policy,
   create or revise a compact entry before the repository change settles.
   Use `sirno_entry_new` for new entries when available.
   Leave generated footer regions untouched.

3. Edit the repository.
   Make the source, test, document, artifact, config, or skill change from the updated entry.
   Keep the implementation narrow.
   When evidence exists, add or refine precise witness blocks around the smallest stable region.
   Do not add placeholder witnesses.
   If evidence supports a related but different claim, create the exact entry for that claim.

4. Keep configuration aligned.
   Prefer MCP tools for routine lake and frost setup or moves.
   Use manual `Sirno.toml` edits only for schema work or comment maintenance the MCP tools cannot express.
   Preserve path rules from `Sirno.toml` and structural field meaning from the active project.
   Add `[repo].members` only for intended witness surfaces.
   Run deterministic config repair when available after manual config edits.

5. Sync public documentation.
   Treat long-form docs outside the configured lake as repository material.
   Keep durable design claims in entries,
   then update public prose from those entries.
   Use the repository's own documentation-writing method when one exists.

## Validation

Run `sirno_lake_render` after lake metadata changes.
Run `sirno_lake_check` in edit mode and review mode before treating the work as complete.
Run direct `sirno_entry_witness` queries for changed evidence.
Run the formatter, tests, and checks that fit the active repository.

If a check is blocked, report the blocker and still validate entry parsing, metadata references,
and witness output as far as the tools allow.
If the current checkout is frozen or immutable, use the configured frost workflow instead of
forcing a write.
Stage only the changed entries and artifacts,
the repository files that actualize them,
and directly related configuration or documentation.
