---
name: Design Doc Writer Skill
desc: An adjacent meta-management skill for maintaining design documents.
category:
  - meta
belongs:
  - agent-skills
---

`design-doc-writer-skill` documents the adjacent meta-management skill
for maintaining design documents.
Here, meta-management means keeping design documents about a project and its method coherent.
The skill is documented in this lake because Sirno skill work can depend on its ideas,
but it is not one of the five packaged Sirno skills.
Its full MCP resource text lives in `.artifacts/design-doc-writer-skill/SKILL.full.md`.
That artifact follows the exact `.agents/skills/design-doc-writer/SKILL.md` document.
`src/mcp.rs` embeds it as `sirno://skills/design-doc-writer`.

The skill applies when an agent edits `DESIGN.md`,
an equivalent design document,
or design prose that needs the same whole-document discipline.
It asks the agent to read the target document before editing,
identify the concepts, terms, and section order that the edit touches,
apply the smallest structural change that improves clarity,
then reread the changed section and its neighbors for flow and overlap.

Its reusable habits are reader evaluation,
conceptual ordering,
declarative precision,
positive definitions,
and whole-document coherence.
A design document should explain the system through ordered concepts and exact terms.
It should define a term before using it to state a rule,
merge sections that describe the same concept at different levels of detail,
and move local details near the concept they constrain.

The prose style is declarative, dry, and precise.
It prefers short main clauses over nested subordination.
It uses an impersonal voice without becoming bureaucratic.
It avoids motivational framing, rhetoric, and defensive rebuttal.
It states the positive rule first when documenting a constraint,
and uses definition by negation only when it prevents a likely confusion.

Sirno skills use this entry as method input when they touch design documents or design entries.
Lake editing folds in its reader-evaluation and design-prose standards.
Skill synthesis may read it as shared method,
but this entry does not render a `.agents/skills/sirno-*` package.
Only the Sirno discipline entries named by `agent-skills` render those packages.
The design-doc-writer resource has no installed wrapper in the Sirno wrapper set.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
