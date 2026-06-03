---
name: Lake Curation Discipline
desc: The agent procedure for auditing and curating an existing Sirno-managed lake.
category:
  - meta
belongs:
  - agent-skills
prerequisite:
  - agent-skills
  - portable-agent-skill-language
  - methodology
  - semantic-locality
  - sirno-witness
---

Lake curation maintains the coherence of an existing Sirno-managed lake.
Its full MCP resource text lives in `.artifacts/lake-curation-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-curator`.
Its packaged wrapper lives in `.artifacts/lake-curation-discipline/SKILL.md`
and renders to `.agents/skills/sirno-curator/SKILL.md`.

Lake curation is the proactive counterpart to repository editing.
The editor skill responds to a request to change repository material.
The curator skill is invoked when no code change drives the work,
but the lake itself needs a pass for clarity, focus, accurate structure,
or witness alignment.

Orient to the project first.
Read repository instructions and `Sirno.toml`.
Call `sirno_status` to surface the lake path, tide blockers, and pending review entries.
Run `sirno_entry_query` to enumerate the lake.
Read the narrative routes first:
introduction, methodology, the README entry,
and any other entries categorized as narrative.
Read the meta entries to learn the project's own principles, vocabulary,
and category conventions before judging any entry against them.

Audit each entry against a rubric.
An entry should state one durable design fact and stay focused enough to read in place.
It should satisfy semantic locality:
the body carries the local meaning,
while links and witnesses add navigation or evidence.
Semantic locality still allows useful lists.
A route, index, roster, or review front door may enumerate children
when that list is part of the entry's local claim.
Flag an entry as a curation candidate when any of these holds:
the body is glossary-thin, defining a term without carrying design pressure;
the body is backlog-thin, naming a work item rather than durable design knowledge;
the body depends on a linked entry or witness to recover its basic semantics;
the body presents a child list as a definition rather than as a route, roster, index, or review front door;
the framing leads with defensive negation instead of positive definition;
an inventory, workflow, comparison, or relationship is buried in prose
that bullets, steps, tables, or a simple diagram would make easier to scan;
another entry already covers the same material;
the entry's role does not match its configured category, belongs, or refines edges;
or the entry sits outside a structural neighborhood that its peers all join.

Discover missing structure.
When one entry's prose claims to make another more concrete,
check that `refines` carries that edge in metadata.
When a family of peers shares a theme,
check that `belongs` places them in the same review neighborhood.
When an entry sits outside a neighborhood that its category peers all join,
the absence is usually an oversight, not a design choice.
A vertical specialization edge and a horizontal review neighborhood can both apply,
but they should not collapse onto the same target.

Identify witness gaps and witness drift.
A witness is a validation link, not a storage mechanism.
For each entry that names a claim a repository region should honor,
run `sirno_entry_witness` to see what is currently bound.
An entry with a code-honoring claim and no witness is a candidate to add one.
A witness pointing at code that no longer asserts the entry's claim
is a candidate to retarget or remove.
Do not add witnesses for entries that hold prose only.

Propose before acting.
A curation pass touches many entries.
Present findings as a punch list before deleting, merging, or restructuring.
Group items by confidence:
high-confidence cleanups, medium-confidence consolidations,
and low-confidence observations the user may want to leave alone.
Ask explicit questions when a choice has more than one defensible answer,
such as where to fold a thin entry
or whether to keep a pure review-neighborhood front door.

Act on the approved items.
Trim and reframe surviving entries,
removing duplication and replacing defensive negation with positive definition.
Preserve useful lists when the entry owns that route, roster, index, or review front door.
Move implementation inventories into narrower entries when they are not part of the entry's local claim.
Fold thin entries into the front-door entries that already cover them;
preserve the durable design fact and drop implementation detail that follows from other claims.
When folding entry A into entry B,
retarget any repository witness sentinels from A to the surviving claim entry
before deleting A,
so the lake stays checkable between commits.

Wire missing witnesses while the curation pass is fresh.
When an entry names a claim that should be inspectable in repository code,
add a witness block around the smallest stable region that supports the claim.
A test that asserts the claim is a strong witness;
a configuration boundary or generated artifact may also qualify.
Skip the witness when the entry holds only prose.

Validate at the review boundary.
Run `sirno_mist_render` after lake metadata changes.
Run `sirno_lake_check` in edit mode and review mode before treating the work as complete.
Run `sirno_entry_witness` on entries whose evidence changed.
If `sirno_status` reports an open tide,
walk workitems with `sirno_tide_status`
and resolve them with `sirno_tide_resolve` or `sirno_tide_unresolve`
once the curation pass is otherwise complete.
If the current checkout is frozen or an entry is immutable,
use `sirno_entry_melt` instead of forcing a write.

Stage narrowly.
One logical change per commit.
Pair sentinel retargets with the entry merges they enable,
or sequence the commits so each one leaves the lake checkable on its own.
Prefer many small commits over one large pass.

The curator proposes, the human disposes.
Deleting an entry, restructuring a refines chain,
or changing a frozen entry are not unilateral acts.
This skill is a maintenance partner for the human reviewer,
not an autonomous rewriter of the lake.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
