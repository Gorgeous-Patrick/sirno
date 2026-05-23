---
name: Actualization Discipline
desc: The agent procedure for shaping repository material from the configured lake.
category:
  - meta
belongs:
  - agent-skills
prerequisite:
  - agent-skills
  - portable-agent-skill-language
  - methodology
  - actualize
  - sirno-witness
  - project-config
refines:
  - actualize
---

Actualization shapes repository material from the configured lake.
Its full MCP resource text lives in `.artifacts/actualization-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-actualizer`.
Its packaged wrapper lives in `.artifacts/actualization-discipline/SKILL.md`
and renders to `.agents/skills/sirno-actualizer/SKILL.md`.

Use this skill for the lake-to-repo direction of repository work.
That covers implementing a claim,
refactoring code toward an entry,
rendering or rewriting a non-lake document from entries,
writing tests for a stated invariant,
updating configuration to match a named boundary,
and adding or refining witness blocks around stable evidence.
When the work is primarily about teaching the lake from repo material,
use the internalizer skill instead.
When a request mixes directions or its direction is unclear,
defer to the front-door editor skill.

The lake is the canonical source of design.
Actualization shapes repository material to match it.
A frozen entry records the user's deliberate decision that the entry has been reviewed
and should stay as truth.
Freeze is not an absolute green light;
it marks information the user wants treated with care.
Read frozen entries as part of the design source.
If actualization seems to require a frozen entry to change,
surface the proposed change to the user for double review
before melting it with `sirno_entry_melt`
or unfreezing the checkout with `sirno_frost_checkout`.

Map the project first.
The configured lake and `Sirno.toml` are read as truth;
the repository is the material to be shaped.
Bind the MCP server to the repository root through `sirno_cwd` when needed.
Read repository instructions and `Sirno.toml`.
If `Sirno.toml` is missing, report that the repository is not Sirno-managed
and prompt the user to start with `sirno init`.
Call `sirno_status` to surface the lake path, frost state, tide blockers,
and pending review entries.
Query the active lake with `sirno_entry_query`,
then read the few candidate entries that govern the request.
Read frozen entries with the same weight as mutable ones;
let the freeze mark guide care rather than skipping.
Follow configured structural fields such as `category`, `belongs`, `prerequisite`, and `refines`.
Use `sirno_entry_witness` to inspect existing evidence before touching repository material.

Confirm the entries name the work.
The entry is authoritative for the design claim, whether frozen or mutable.
If an existing entry states the right claim, proceed under that entry.
If a small, intent-driven entry is needed and the durable fact is already clear,
create or revise the compact entry inline with `sirno_entry_new` before the repository
change settles.
If the work exposes a durable design fact that is not yet named in the lake
and the fact deserves its own deliberation,
hand off to the internalizer skill,
then return to actualization once the entry exists.
Leave generated footer regions untouched.

Edit the repository.
The entry holds the claim;
the source, test, document, artifact, configuration, or skill change is shaped from it.
Keep the implementation narrow and aligned with the entry claim.
Do not drift into adjacent cleanup that the entry does not name.
When repository evidence exists,
add or refine precise witness blocks around the smallest stable region
that supports the entry.
Do not add placeholder witnesses.
If evidence supports a related but different claim,
create the exact entry for that claim through the internalizer skill,
then witness it.

Maintain project configuration as part of the same lake-to-repo workflow.
The lake entries that govern project shape are authoritative;
`Sirno.toml` is the operational configuration in the repository
that should remain consistent with them.
Prefer MCP tools such as `sirno_lake_move`, `sirno_frost_*`, and `sirno_entry_artifact_*`
for routine lake, frost, and artifact moves.
Use manual `Sirno.toml` edits only for schema work or comment maintenance the MCP tools
cannot express,
then run deterministic config repair when available.
Preserve path rules from `Sirno.toml` and structural field meaning from the active project.
Add `[repo].members` paths only when they are intended witness surfaces.

Sync public documentation from the lake.
The entries hold the durable design claims;
the published document is shaped from them.
Treat long-form docs outside the configured lake as repository material.
Render or rewrite them from the entries that name those claims.
Use the repository's own documentation-writing method when one exists;
otherwise fall back to `sirno://skills/design-doc-writer`.

Validate at the review boundary.
The lake remains the source of truth;
the structural checks confirm that it still resolves cleanly after the edit.
Run `sirno_lake_render` after lake metadata changes.
Run `sirno_lake_check` in edit mode and review mode before treating the work as complete.
Run `sirno_entry_witness` against any entry whose evidence changed.
Run the formatter, tests, and checks that fit the active repository.

If `sirno_status` reports an open tide,
walk workitems with `sirno_tide_status`,
then use `sirno_tide_resolve` or `sirno_tide_unresolve` rather than ignoring the blocker.
If the current checkout is frozen or an entry is immutable, never force a write.
Surface the proposed change to the user for double review,
then use `sirno_frost_checkout`, `sirno_entry_melt`, or the project's frost workflow
once the user agrees.
If a check is blocked, report the blocker and still validate entry parsing,
metadata references, and witness output as far as the tools allow.

Stage only the changed entries and artifacts,
the repository files that actualize them,
and directly related configuration or documentation.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
