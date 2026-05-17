---
name: Lake Editing Discipline
desc: The agent procedure for editing Sirno Lake entries.
category:
  - meta
belongs:
  - agent-skills
---

Lake editing follows a fixed procedure so the lake stays precise and reviewable.

Read first.
Read repository instructions, `Sirno.toml`, the configured monograph when present,
and the existing lake.
Decide design authority before changing anything; see `design-source-authority`.
Inspect the current Sirno CLI before assuming which commands exist.

Map before editing.
For each candidate entry, know its id, name, desc, structural fields, and witness status.
Use `sirno query` to find concepts and neighborhoods,
and `sirno rg` for literal text inside entries.
Read a matched entry before rewriting it; do not edit from isolated match lines.

Create through the tool.
Create missing entries with the current entry-creation command so id validation and scaffolding
are correct.
Then expand or revise the body with direct, reader-friendly prose.
Choose `category`, `belongs`, and `refines` by their own entries,
and leave a structural field out when it is merely decorative.

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
If review-mode checks fail only because local editor or tool directories live inside the lake,
preserve those files unless the user asks to remove them,
report the blocker,
and still validate entry parsing and metadata references as far as possible.

Stage narrowly when committing.
Stage the configured lake, the config change that points to it, and directly related
documentation, and leave unrelated code or generated editor state alone.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
