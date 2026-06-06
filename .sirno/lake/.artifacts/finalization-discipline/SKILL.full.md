---
name: sirno-finalizer
description: >-
  Finalize reviewed Sirno-managed work by preserving session continuity, reviewing design fluency,
  resolving Tide, updating Anchor, staging scoped changes, and committing by default.
---

# Sirno Finalizer

## Purpose

Use this skill at a Sirno review boundary.
It accepts the current waterline after the active change has been shaped by editing skills.
It refreshes Sirno and Git state, reviews the larger design shape,
validates the lake, clears Tide, updates Anchor, stages narrowly, and commits by default.

The finalizer is not a substitute for editing.
If the change still needs source, lake, witness, artifact, or skill-synthesis work,
return to the editor, actualizer, internalizer, curator, or synthesizer first.
Skip the commit only when the user explicitly asks to validate only,
update Anchor only, or leave the work staged.

This full skill text is served as `sirno://skills/sirno-finalizer`.
Apply it to the project currently bound through Sirno MCP.

## Session Stance

Treat the current agent session as continuous and safe review context.
The agent may rely on its own reads, edits, reasoning, and validations from this session
unless current repository state or a fresh check contradicts them.
A context transition, resume, or finalizer invocation does not by itself make the work suspect.
Reorient enough to scope current files and entries,
then finish the review boundary.

## Project Binding

Bind the MCP server to the repository before calling project tools.
Call `sirno_cwd` with the repository root when the server may not already be there.
Project tools resolve `Sirno.toml` from the server current working directory on each call.
Call `sirno_cwd` again before switching projects in the same server process.

## Workflow

1. Scope the active change.
   Read repository instructions and `Sirno.toml`.
   If `Sirno.toml` is missing, report that the repository is not Sirno-managed.
   Call `sirno_status` to see lake path, mist state, Tide state, and review checks.
   Inspect `git status --short` before staging anything.
   Identify the files and entries that belong to the user's current request.
   Leave unrelated dirty work untouched.

2. Review the design as a reader.
   Read the changed entries and the entries that govern the review boundary.
   At minimum, read the relevant changed entries plus `lake-review`, `anchor`, `tide`,
   and `versioning`.
   When skill packages changed, read `agent-skills` and `skill-synthesis-discipline`.
   When repository evidence changed, use `sirno_entry_witness` to inspect the affected entries.
   Check that the design lives at the right level,
   uses structural links for review, navigation, or accountability,
   and preserves semantic locality.
   Read the affected route through the configured lake as a person would:
   definitions before rules,
   broad entries before refinements,
   routes that say what they are for,
   and no stale, redundant, or conflicting neighboring prose.
   For broad or meta changes,
   widen the reader pass until the affected route is fluent end to end.
   If the design is awkward, transitional, or weaker in the larger picture,
   return to the appropriate editing skill before acceptance.

3. Validate the waterline.
   Run `sirno_mist_render` after reservoir metadata changes.
   Run `sirno_lake_check` in edit mode and review mode.
   Run `sirno_mist_status` and require a clean editable mist before accepting Anchor.
   Run `sirno_entry_witness` against entries whose evidence changed.
   Run the formatter, tests, and checks that fit the changed repository material.
   For Rust changes in the Sirno source repository, run `cargo fmt` and `cargo clippy`.

4. Clear Tide and update Anchor.
   If Tide is open, inspect workitems with `sirno_tide_status`.
   Resolve only obligations explained by the reviewed current change.
   Use inference when the ripple and neighbor changed together and the shared review is clear.
   Use explicit neighbor or workitem resolution when a reviewed ripple affects older neighbors.
   If a workitem points to unrelated user work, stale projection state, or unclear design impact,
   stop and report the blocker instead of forcing acceptance.
   Call `sirno_anchor_update` only after review checks pass, mist status is clean,
   and Tide is clear.
   Then call `sirno_anchor_status` and confirm the anchor is current.
   If Anchor refuses the update, inspect the reported blocker and return to the relevant step.

5. Stage, commit, and confirm.
   Stage only files that belong to the accepted current change:
   changed lake entries, entry artifacts, repository material, generated skill wrappers,
   MCP resource constants, directly related configuration, and `.sirno/anchor.toml`.
   Do not stage misty-lake projection files unless they are intentional repository material.
   Re-check `git status --short` after staging.
   Treat finalization as an acceptance and commit boundary.
   Create a commit after validation, Tide review, Anchor update, and scoped staging succeed.
   Skip the commit only when the user explicitly asks to validate only,
   update Anchor only, or leave the work staged.
   Use the repository commit-message convention.
   Prefer one logical commit for the accepted change.
   If the finalization discovered unrelated changes, leave them unstaged and report them.
   After the commit, confirm a clean worktree,
   current Anchor,
   clear Tide,
   and clean mist status.
   Report the commit hash and the validations that passed.

## Failure Paths

If `Sirno.toml` is missing,
report that the repository is not Sirno-managed and do not continue.

If mist status is dirty,
render, intake, or report the exact projection blocker before updating Anchor.

If Tide cannot be resolved from the reviewed change,
leave Anchor unchanged and report the unresolved entries or workitems.

If tests or checks fail,
do not update Anchor or commit until the failure is fixed or the user explicitly accepts the risk.

If commit creation fails,
leave the staged state visible and report the command failure.
