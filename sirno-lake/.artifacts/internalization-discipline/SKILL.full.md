---
name: sirno-internalizer
description: >-
  Record durable design facts from repository material into the configured lake.
---

# Sirno Internalizer

## Purpose

Use this skill for the repository-to-lake direction of repository work.
Record durable design facts from source, tests, generated artifacts, configuration,
or non-lake documentation into the active project's Sirno Lake.

Internalization names a boundary, invariant, representation, behavior, contract, or policy
that future work should reason through.
When the work primarily shapes repository material from an existing entry,
use `sirno://skills/sirno-actualizer`.
When the direction is mixed or unclear,
use `sirno://skills/sirno-editor`.

This full skill text is served as `sirno://skills/sirno-internalizer`.
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
   and pending review entries.
   Query the active lake with `sirno_entry_query` to locate the candidate neighborhood.
   Read frozen entries before declaring an observation new.

2. Identify the durable fact.
   The repository supplies evidence.
   The fact becomes canonical only when an entry names it.
   Reject changelog material; commit history can carry the sequence of edits.
   Keep facts that reshape future reasoning:
   representations, invariants, boundaries, behaviors, contracts, and policies.

3. Choose the entry handle.
   If a mutable entry already names the right claim,
   revise it in place to record the sharper fact.
   If a frozen entry already covers the fact,
   surface any proposed revision to the user for double review before melting it.
   If the fact belongs to a new boundary,
   create a compact entry with `sirno_entry_new`.
   If an existing entry's name no longer fits the sharper fact,
   use `sirno_entry_rename`.
   Leave generated footer regions untouched.

4. Write and connect the entry.
   State the fact directly and keep the body compact.
   Apply semantic locality:
   the entry should remain meaningful when read without its neighbors,
   and links should help navigation rather than supply the basic semantics.
   Use paragraphs for continuous claims.
   Use bullets, numbered steps, tables, or simple diagrams
   when they make inventories, workflows, comparisons, or relationships easier to scan.
   Attach link relations only when they improve navigation, review, or accountability.
   When repository evidence already exists,
   add or refine a precise witness block around the smallest stable region that supports the entry.
   Do not add placeholder witnesses.
   If the entry holds prose only or its evidence is not yet in the repository,
   leave it unwitnessed and let the actualizer skill wire evidence later.

5. Migrate durable claims out of non-lake documents when that is the work.
   Treat long-form documents outside the configured lake as repository material.
   Internalize first,
   then let the actualizer skill regenerate the published form from the new entries.

## Validation

Run `sirno_lake_render` after lake metadata changes.
Run `sirno_lake_check` in edit mode and review mode before treating the work as complete.
Run `sirno_entry_witness` against any entry whose evidence references changed.
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

Stage only the changed entries and artifacts.
If actualization should follow to update repository material from the new entries,
sequence the work so each commit leaves the lake checkable on its own.
