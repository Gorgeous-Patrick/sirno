---
desc: The agent procedure for accepting, staging, and committing a reviewed Sirno change.
name: Finalization Discipline
category:
  - meta
belongs:
  - agent-skills
prerequisite:
  - agent-skills
  - repository-editing-discipline
  - lake-review
  - versioning
  - portable-agent-skill-language
---

Finalization completes a reviewed Sirno-managed repository change.
Its full MCP resource text lives in `.artifacts/finalization-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-finalizer`.
Its packaged wrapper lives in `.artifacts/finalization-discipline/SKILL.md`
and renders to `.agents/skills/sirno-finalizer/SKILL.md`.

The finalizer is the review-boundary counterpart to repository editing.
The editor shapes the change under named entries.
The finalizer verifies that the waterline is coherent,
walks Tide obligations,
updates Anchor when the waterline has been reviewed,
and commits the scoped repository change when the user requested a commit.

Start from the active project.
Read repository instructions, `Sirno.toml`, and the entries that govern the current work.
Call `sirno_status` to see mist state, Tide state, and review checks.
Inspect `git status` before staging anything.
Unrelated dirty files belong to the user and stay outside the finalization set.

Validate before acceptance.
Run mist render when reservoir metadata changed.
Run lake checks in edit and review modes.
Run witness checks for entries whose evidence changed.
Run the repository formatter, tests, or checks that fit the changed material.
For Rust changes in this repository, that means `cargo fmt` and `cargo clippy` at minimum.

Walk Tide rather than bypassing it.
If Tide reports open workitems,
read the workitems and resolve only obligations that the current review explains.
Inference is appropriate when the changed ripple and neighbor were reviewed together.
If an obligation points to unrelated work, stale projection state, or unclear design impact,
stop and report the blocker instead of forcing acceptance.

Update Anchor only after review checks pass, mist state is clean, and Tide is clear.
Anchor records the accepted lake baseline.
It should be staged with the reservoir, control files, and repository material that made the lake acceptable.

Commit only when the user asked for a commit or explicitly invoked finalization as a commit boundary.
Stage narrowly:
include changed entries, entry artifacts, repository files, configuration, generated skill wrappers,
MCP resource constants, and `.sirno/anchor.toml` only when they belong to the current change.
Use the repository commit-message convention.
After committing, confirm a clean worktree, current Anchor, clear Tide, and clean mist status.

If finalization cannot complete,
report the exact blocker and leave the worktree staged only when staging was already part of the
requested operation.
