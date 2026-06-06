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
It treats the current agent session as continuous reviewed context:
the agent may rely on its own reads, edits, and validations from this session
unless the current repository state contradicts them.
It refreshes Sirno status and Git state before acceptance,
but it should not restart work or invent interruption risk merely because it is finalizing.

Acceptance is a design judgment before it is a mechanical closeout.
Read the changed entries in their design neighborhood.
Check that the change lives at the right level,
uses structural links for review, navigation, or accountability,
and preserves semantic locality.
Read the lake as a person would:
definitions should precede rules,
broad entries should lead cleanly to refinements,
routes should say what they are for,
and neighboring prose should not overlap or fight the local claim.
If the design is awkward, transitional, or less fluent after the change,
return to the appropriate editing skill before accepting the waterline.

After that reader pass,
validate the lake and changed repository material,
walk Tide obligations that the reviewed change explains,
update Anchor only when checks pass and Tide is clear,
then stage and commit the scoped change by default.
Unrelated dirty files belong to the user and stay outside the finalization set.
If finalization cannot complete,
report the exact blocker and leave the worktree staged only when staging was already requested.
