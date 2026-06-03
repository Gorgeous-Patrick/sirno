---
name: Internalization Discipline
desc: The agent procedure for recording durable design facts from repository material into the lake.
category:
  - meta
belongs:
  - agent-skills
prerequisite:
  - agent-skills
  - portable-agent-skill-language
  - methodology
  - semantic-locality
  - internalize
  - sirno-witness
refines:
  - internalize
---

Internalization records durable design facts from repository material into the configured lake.
Its full MCP resource text lives in `.artifacts/internalization-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-internalizer`.
Its packaged wrapper lives in `.artifacts/internalization-discipline/SKILL.md`
and renders to `.agents/skills/sirno-internalizer/SKILL.md`.

Use this skill for the repo-to-lake direction of repository work.
That covers naming a newly stable boundary, invariant, representation, behavior, contract,
or policy;
sharpening an entry after a refactor reveals a clearer local design;
folding a parser invariant or storage edge into the lake while the work is fresh;
and migrating a durable design claim out of a non-lake document into an entry.
When the work is primarily about shaping repository material from the lake,
use the actualizer skill instead.
When a request mixes directions or its direction is unclear,
defer to the front-door editor skill.

The lake remains the canonical source of design throughout internalization.
The repository supplies evidence of a durable fact;
the fact becomes canonical only when an entry names it.
A frozen entry records the user's deliberate decision that the entry has been reviewed
and should stay as truth.
Freeze is not an absolute green light;
it marks information the user wants treated with care.
Read frozen entries before declaring an observation new.
If internalization seems to require a frozen entry to change,
surface the proposed change to the user for double review
before melting it with `sirno_entry_melt`,
rather than writing a parallel entry past it.

Orient to the project first.
The existing lake, including frozen entries, is the canonical record of prior design;
the repository is the evidence to be considered.
Bind the MCP server to the repository root through `sirno_cwd` when needed.
Read repository instructions and `Sirno.toml`.
If `Sirno.toml` is missing, report that the repository is not Sirno-managed
and prompt the user to start with `sirno init`.
Call `sirno_status` to surface the lake path, tide blockers, and pending review entries.
Query the active lake with `sirno_entry_query` to locate the candidate neighborhood
for the durable fact.

Identify the durable design fact.
The fact is observed from the repository but does not become canonical until an entry names it.
Read any frozen entry that might already cover it before declaring the fact new;
the freeze marks user-reviewed truth that deserves care.
Name what the repository now knows that the lake should know too:
a representation, an invariant, a boundary, a behavior, a contract, or a policy.
Reject changelog material.
The commit history can carry the sequence of edits.
The lake should carry the design that survived the edit.
If a change was exploratory and later discarded,
it usually does not deserve internalization.
If a change reshapes how future work should reason,
it should be internalized while the reason is still clear.

Choose the entry handle.
Existing entries are authoritative for the claims they already record.
A frozen entry that already names the fact remains user-reviewed truth;
surface any proposed revision to the user for double review
before melting it with `sirno_entry_melt`,
rather than writing a parallel entry past it.
Read candidate entries and walk `belongs`, `prerequisite`, and `refines`
to find the closest existing neighbor.
If a mutable entry names the right claim,
revise it in place to record the sharper design fact.
If the fact belongs to a new boundary,
create a compact entry with `sirno_entry_new`.
If an existing entry's name no longer fits the sharper fact,
use `sirno_entry_rename`.
If the fact deserves its own deliberation distinct from a near-enough entry,
create the new entry rather than overloading the old one.
Leave generated footer regions untouched.

Write the entry prose.
On completion, the entry becomes the canonical statement of the design fact.
State the fact directly.
Apply semantic locality:
the entry should remain meaningful when read without its neighbors,
and links should help navigation rather than supply the basic semantics.
Routes, indexes, rosters, and review front doors may enumerate children
when that list is part of the local claim.
Say what the list is for so it does not masquerade as the semantic contract.
Keep the body small enough to read in place and durable enough to survive
the edit that made it useful.
Use paragraphs for continuous claims.
Use bullets, numbered steps, tables, or simple diagrams when the fact is easier to scan
as an inventory, workflow, comparison, or relationship.
Prefer positive definition over defensive negation.
Attach `category`, `belongs`, `prerequisite`, and `refines` only when they improve
navigation, review, or accountability.

Connect to evidence without manufacturing it.
The entry holds the claim;
the witness names the repository region that should demonstrate it.
When repository evidence already exists for the claim,
add or refine a precise witness block around the smallest stable region that supports
the entry.
A test, configuration boundary, or generated artifact may qualify.
Do not add placeholder witnesses.
If the entry holds prose only or its evidence is not yet in the repo,
leave it unwitnessed and let the actualizer skill wire evidence later.

Migrate durable claims out of non-lake documents when that is the work.
Long-form documents outside the configured lake are repository material;
the entry becomes canonical for the durable claim after migration.
Internalize first, then let the actualizer skill regenerate the published form.

Validate at the review boundary.
The reservoir remains the source of truth;
the structural checks confirm that it still resolves cleanly after the entries change.
Run `sirno_mist_render` after reservoir metadata changes.
Run `sirno_lake_check` in edit mode and review mode before treating the work as complete.
Run `sirno_entry_witness` against any entry whose evidence references changed.

If `sirno_status` reports an open tide,
walk workitems with `sirno_tide_status`,
then use `sirno_tide_resolve` or `sirno_tide_unresolve` rather than ignoring the blocker.
If the current checkout is frozen or an entry is immutable, never force a write.
Surface the proposed change to the user for double review,
then use `sirno_entry_melt` once the user agrees.
If a check is blocked, report the blocker and still validate entry parsing,
metadata references, and witness output as far as the tools allow.

Stage only the changed entries and artifacts.
If actualization should follow to update repository material from the new entries,
sequence the work so each commit leaves the lake checkable on its own.
