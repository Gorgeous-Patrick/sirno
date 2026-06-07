---
desc: The agent procedure for codebase changes that start in the Sirno Lake and keep documentation synced.
name: Repository Editing Discipline
category:
  - meta
belongs:
  - agent-skills
prerequisite:
  - agent-skills
  - portable-agent-skill-language
  - methodology
  - semantic-locality
  - actualize
  - witness
  - project-config
refines:
  - methodology
---

Repository editing is the default agent procedure for changing a Sirno-managed repository.
Its static full-resource template lives in
`.artifacts/repository-editing-discipline/SKILL.full.template.md`
and is rendered by `src/mcp.rs` as `sirno://skills/sirno-editor`.
Its packaged wrapper lives in `.artifacts/repository-editing-discipline/SKILL.md`
and renders to `.agents/skills/sirno-editor/SKILL.md`.

Start every repository edit in the configured lake.
Read repository instructions, `Sirno.toml`, and the entries that govern the requested work.
If `Sirno.toml` is missing, report that the repository is not currently Sirno-managed,
then prompt the user to start with `sirno init`.
Call `sirno_status` early to surface the lake path, tide blockers, and pending review entries.
Use `sirno_entry_query` for discovery,
follow the active project's configured structural relations,
and inspect existing evidence with `sirno_entry_witness` before editing repository material.
This applies to source, tests, generated artifacts, skill packages, README files,
configuration, and design documents outside the configured lake.

Name or revise the durable design fact before the repository change settles.
If an entry already states the right claim, keep the code change under that entry.
If the work reveals a new boundary, invariant, representation, or behavior,
create or revise the compact entry first.
Apply semantic locality when creating or revising entries.
The body should state the local meaning in place,
and any route, index, roster, or review front door should say what the list is for.
Keep the semantic contract separate from the current route through children.
Use configured structural relations only when they improve navigation, review, or accountability.
Leave generated footer regions untouched.

Actualize from the updated entries into repository material.
Keep the implementation narrow and aligned with the entry claim.
When repository evidence exists, add or refine precise witness blocks around the smallest stable
code, test, configuration, or artifact region that supports the entry.
Do not create placeholder witnesses.
When evidence supports a related but different claim, create the exact entry for that claim.

Maintain project configuration as part of the same workflow.
Prefer MCP tools for routine lake movement, Anchor checks, and Anchor updates.
When manual `Sirno.toml` edits are needed, preserve schema comments and path rules from
`Sirno.toml` and the active project,
then run deterministic config repair when available.
Only add repository members when those paths are intended witness surfaces.

Sync long-form public documentation from the entries that name its design claims.
Use the repository's own documentation-writing method when one exists;
otherwise fall back to `sirno://skills/design-doc-writer`.
Choose paragraphs, bullets, tables, numbered steps, or simple diagrams according to
what makes the design easiest for a human co-worker to scan and review.

Validate at the review boundary.
Run `sirno_mist_render` after reservoir metadata changes,
then run `sirno_lake_check` in edit and review modes.
Run direct witness queries for changed evidence.
If `sirno_status` reports an open tide,
walk workitems with `sirno_tide_status` and resolve them with
`sirno_tide_resolve` or `sirno_tide_unresolve` rather than ignoring the blocker.
If the current checkout is frozen or an entry is immutable,
use `sirno_entry_melt` instead of forcing a write.
If checks are blocked, report the blocker and still validate entry parsing,
metadata references, and witness output as far as the tools allow.

Stage narrowly when committing.
Stage the changed entries and artifacts,
the repository files that actualize them,
and directly related config or documentation.
Leave unrelated work untouched.
