---
name: sirno-finalizer
description: >-
  Finalize reviewed Sirno-managed repository work by validating the lake, resolving Tide,
  updating Anchor, staging scoped changes, and committing by default.
---

# Sirno Finalizer

## Purpose

Use this skill at a Sirno review boundary.
It completes reviewed repository work by validating the active lake,
walking Tide obligations,
updating Anchor after the waterline is accepted,
and creating a scoped Git commit by default.

The finalizer is not a substitute for repository editing.
Use the editor, actualizer, internalizer, curator, or synthesizer skill for the work itself.
Use the finalizer when the remaining task is to accept and commit that work.
If the user wants validation, Anchor update, or staging without a commit,
they should say so explicitly.

Repository material covers source, tests, generated artifacts, configuration,
README files, design documents outside the configured lake, witness blocks, and skill packages.
Finalization keeps those files aligned with the lake and Anchor baseline.

This full skill text is served as `sirno://skills/sirno-finalizer`.
Apply it to the project currently bound through Sirno MCP.

## Project Binding

Bind the MCP server to the repository before calling project tools.
Call `sirno_cwd` with the repository root when the server may not already be there.
Project tools resolve `Sirno.toml` from the server current working directory on each call.
Call `sirno_cwd` again before switching projects in the same server process.

## Workflow

1. Orient to the active change.
   Read repository instructions and `Sirno.toml`.
   Call `sirno_status` to surface the lake path, mist state, Tide blockers, and review checks.
   Inspect `git status` before staging anything.
   Identify the files and entries that belong to the user's current request.
   Leave unrelated dirty work untouched.

2. Read the governing entries.
   Read the entries that explain the current change and the review boundary.
   At minimum, read the relevant changed entries plus `lake-review`, `sirno-anchor`,
   `sirno-tide`, and `versioning`.
   When skill packages changed, read `agent-skills` and `skill-synthesis-discipline`.
   When repository evidence changed, use `sirno_entry_witness` to inspect the affected entries.

3. Validate before acceptance.
   Run `sirno_mist_render` after reservoir metadata changes.
   Run `sirno_lake_check` in edit mode and review mode.
   Run `sirno_mist_status` and require a clean editable mist before accepting Anchor.
   Run the formatter, tests, and checks that fit the changed repository material.
   For Rust changes in this repository, run `cargo fmt` and `cargo clippy`.

4. Review Tide deliberately.
   If Tide is open, inspect workitems with `sirno_tide_status`.
   Resolve only obligations explained by the reviewed current change.
   Use inference when the ripple and neighbor changed together and the shared review is clear.
   Use explicit neighbor or workitem resolution when a reviewed ripple affects older neighbors.
   If a workitem points to unrelated user work, stale projection state, or unclear design impact,
   stop and report the blocker instead of forcing acceptance.

5. Update Anchor.
   Call `sirno_anchor_update` only after review checks pass, mist status is clean,
   and Tide is clear.
   Then call `sirno_anchor_status` and confirm the anchor is current.
   If Anchor refuses the update, inspect the reported blocker and return to the relevant step.

6. Stage narrowly.
   Stage only files that belong to the accepted current change:
   changed lake entries, entry artifacts, repository material, generated skill wrappers,
   MCP resource constants, directly related configuration, and `.sirno/anchor.toml`.
   Do not stage misty-lake projection files unless they are intentional repository material.
   Re-check `git status --short` after staging.

7. Commit by default.
   Treat finalization as an acceptance and commit boundary.
   Create a commit after validation, Tide review, Anchor update, and scoped staging succeed.
   Skip the commit only when the user explicitly asks to validate only,
   update Anchor only, or leave the work staged.
   Use the repository commit-message convention.
   Prefer one logical commit for the accepted change.
   If the finalization discovered unrelated changes, leave them unstaged and report them.

8. Confirm the final state.
   After the commit, confirm a clean worktree,
   current Anchor,
   clear Tide,
   and clean mist status.
   Report the commit hash and the validations that passed.

## Failure Paths

If `Sirno.toml` is missing,
report that the repository is not Sirno-managed and do not continue.

If mist status is dirty,
render, intake, or report the exact projection blocker before Anchor update.

If Tide cannot be resolved from the reviewed change,
leave Anchor unchanged and report the unresolved entries or workitems.

If tests or checks fail,
do not update Anchor or commit until the failure is fixed or the user explicitly accepts the risk.

If commit creation fails,
leave the staged state visible and report the command failure.

## Stance

Finalization is an acceptance step.
It should be calm, scoped, and explicit about what is being accepted.
The lake and repository history should end in the same state:
reviewed entries, current Anchor, clear Tide, clean mist, and a Git commit when requested.
