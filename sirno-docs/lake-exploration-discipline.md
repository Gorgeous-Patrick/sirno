---
name: Lake Exploration Discipline
desc: The agent procedure for exploring a Sirno-managed repository from the lake.
category:
  - meta
belongs:
  - agent-skills
---

Exploration reads a Sirno-managed repository from the lake outward.
Its full MCP resource text lives in `.artifacts/lake-exploration-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-explorer`.
Its packaged wrapper lives in `.artifacts/lake-exploration-discipline/SKILL.md`
and renders to `.agents/skills/sirno-explorer/SKILL.md`.

The lake is the project map and witness blocks are the evidence for entry claims.
The goal is a small, grounded route: entry ids, why they matter, witness locations,
and the code or docs they point to.

Follow a fixed order.
Locate the active `Sirno.toml` and read its lake, structural, and repo settings.
Query the lake, starting vague for discovery and using `--has` when a structural field is known,
and read the `desc` field before narrowing.
Read the few highest-signal entries and follow their structural fields.
Ask Sirno for evidence with `sirno witness ENTRY_ID --full`.
Inspect witnessed regions and nearby code, using `sirno rg` inside the lake and plain `rg` for
repository code.
Synthesize the route last.

Treat a witness as evidence, not proof.
A witness says where to inspect a claim; it does not show the code is correct.
When a witness is broad, read it once, then narrow to the smallest relevant function, test, or
config stanza, and note if it would benefit from splitting.
When an entry has no witness, check related entries, search for the id and key terms,
and state whether the result is documentation-only, unwitnessed, or not found.

Keep the route narrow.
Avoid reading the whole lake or repository unless the question asks for a survey.
Do not add or edit witness blocks while exploring;
switch to the witness or editor procedure when the task changes from reading to changing.

Report grounded findings.
A good result names the entries consulted,
the descriptions that shaped the route,
the witness files and line ranges,
the code symbols or docs inspected,
what is known, inferred, and still uncertain,
and a useful next inspection step.
If checks fail, report the blocker and continue with evidence that can still be inspected safely.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
