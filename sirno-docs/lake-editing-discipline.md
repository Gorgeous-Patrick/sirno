---
name: Lake Editing Discipline
desc: The agent procedure for editing Sirno Lake entries.
category:
  - meta
belongs:
  - agent-skills
---

Lake editing follows a fixed procedure so the lake stays precise and reviewable.
Its full MCP resource text lives in `.artifacts/lake-editing-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-editor`.
Its packaged wrapper lives in `.artifacts/lake-editing-discipline/SKILL.md`
and renders to `.agents/skills/sirno-editor/SKILL.md`.

Read first.
Read repository instructions, `Sirno.toml`, relevant repository material, and the existing lake.
Decide design authority before changing anything; see `design-source-authority`.
Inspect the current Sirno MCP tools before assuming which operations exist.

Use the config-writer skill when the design change requires `Sirno.toml` edits.
Lake editing may change entries that describe config,
but config-writing rules live in `config-writing-discipline`.

Map before editing.
For each candidate entry, know its id, name, desc, structural fields, and witness status.
Use `sirno_entry_query` to find concepts and neighborhoods,
and `sirno_entry_rg` for literal text inside entries.
Read a matched entry before rewriting it; do not edit from isolated match lines.

Create through the tool.
Create missing entries with `sirno_entry_new` so id validation and scaffolding are correct.
Then expand or revise the body with direct, reader-friendly prose.
When editing design documents or design entries,
use the repository's own design-document skill or documented manner first.
If none exists, default to the discipline in `sirno://skills/design-doc-writer`,
documented by `design-doc-writer-skill`:
read the whole design route,
order concepts by dependency and scope,
write declarative, dry, precise prose,
merge avoidable overlap,
and evaluate whether each paragraph carries one idea.
Choose `category`, `belongs`, and `refines` by their own entries,
and leave a structural field out when it is merely decorative.
Use section headings only when they frame the material that follows.
Do not leave a heading empty by placing another heading, diagram, or list directly under it.

Distinguish a principle from its application.
A meta-level principle states how the project should be understood or developed.
Applying it produces structural facts:
which entries a `category` names, which `belongs` neighborhood an entry joins.
Those facts live in metadata and generated footers, not in new entries.
Create an entry only when it gives future work a handle
that the principle and its structural edges do not already provide.

Leave generated footers untouched.
After metadata stabilizes, run render maintenance,
then run the editing-mode structural check and the review-mode check.

Validation can be partly blocked.
If the entry is frozen or a checkout is immutable,
use the configured Frost workflow before editing instead of forcing a write.
If review-mode checks fail only because local editor or tool directories live inside the lake,
preserve those files unless the user asks to remove them,
report the blocker,
and still validate entry parsing and metadata references as far as possible.
If a tool named by an old skill is missing,
inspect the current MCP tool list and use the closest current tool only when its behavior is clear.
If authored metadata, references, or generated-footer freshness fail,
fix the lake before treating the edit as complete.

Stage narrowly when committing.
Stage the configured lake, the config change that points to it, and directly related
documentation, and leave unrelated code or generated editor state alone.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
