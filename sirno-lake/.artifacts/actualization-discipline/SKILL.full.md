---
name: sirno-actualizer
description: >-
  Shape repository material from the configured lake when the governing design entry is clear.
---

# Sirno Actualizer

## Purpose

Use this skill for the lake-to-repository direction of repository work.
Shape source, tests, generated artifacts, configuration, or non-lake documentation from the
active project's Sirno Lake.

Actualization starts after the relevant entry is known or can be named compactly.
When the work primarily records a repository fact back into the lake,
use `sirno://skills/sirno-internalizer`.
When the direction is mixed or unclear,
use `sirno://skills/sirno-editor`.

This full skill text is served as `sirno://skills/sirno-actualizer`.
Apply it to the project currently bound through Sirno MCP.

## Project Binding

Bind the MCP server to the repository before calling project tools.
Call `sirno_cwd` with the repository root when the server may not already be there.
Project tools resolve `Sirno.toml` from the server current working directory on each call.
Call `sirno_cwd` again before switching projects in the same server process.

## Workflow

1. Map the design source.
   Read repository instructions and `Sirno.toml`.
   If `Sirno.toml` is missing, report that the repository is not Sirno-managed
   and prompt the user to start with `sirno init`.
   Call `sirno_status` to surface the lake path, tide blockers,
   and pending review entries.
   Query the active lake with `sirno_entry_query`,
   then read the few entries that govern the requested work.
   Read frozen entries with the same weight as mutable ones.
   Use `sirno_entry_witness` to inspect existing evidence before touching repository material.

2. Confirm the entry handle.
   If an existing entry states the right claim, work under that entry.
   If a small entry is needed and the durable fact is clear,
   create or revise it before the repository change settles.
   If the work exposes a new fact that needs separate deliberation,
   hand off to the internalizer skill and return once the entry exists.
   Leave generated footer regions untouched.

3. Shape repository material.
   Keep the edit narrow and aligned with the entry claim.
   Update source, tests, configuration, skill artifacts, or public prose only as the entry requires.
   When repository evidence exists,
   add or refine precise witness blocks around the smallest stable region that supports the entry.
   Do not add placeholder witnesses.
   If evidence supports a related but different claim,
   create the exact entry for that claim through the internalizer skill.

4. Keep adjacent surfaces aligned.
   Prefer MCP tools such as `sirno_lake_move`, `sirno_anchor_status`, `sirno_anchor_check`,
   `sirno_anchor_update`, and `sirno_entry_artifact_*`
   for routine lake movement, Anchor checks, Anchor updates, and artifact moves.
   Use manual `Sirno.toml` edits only for schema or comment work the MCP tools cannot express.
   Treat long-form documents outside the configured lake as repository material.
   Render or rewrite them from the entries that name their design claims.

## Validation

Run `sirno_lake_render` after lake metadata changes.
Run `sirno_lake_check` in edit mode and review mode before treating the work as complete.
Run `sirno_entry_witness` against any entry whose evidence changed.
Run the formatter, tests, and checks that fit the active repository.

If `sirno_status` reports an open tide,
walk workitems with `sirno_tide_status`,
then use `sirno_tide_resolve` or `sirno_tide_unresolve` rather than ignoring the blocker.
If the current checkout is frozen or an entry is immutable,
surface the proposed change to the user for double review,
then use `sirno_entry_melt`
once the user agrees.

If a check is blocked, report the blocker and still validate entry parsing,
metadata references, and witness output as far as the tools allow.

Stage only the changed entries and artifacts,
the repository files that actualize them,
and directly related configuration or documentation.
