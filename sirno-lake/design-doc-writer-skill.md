---
name: Design Doc Writer Skill
desc: The default design-document method Sirno ships as a safety-net MCP resource.
category:
  - meta
belongs:
  - agent-skills
prerequisite:
  - agent-skills
---

`design-doc-writer-skill` is the default design-document method
Sirno provides as a safety-net guideline.
Sirno ships it as the MCP resource `sirno://skills/design-doc-writer`.
There is no installed `.agents/skills` wrapper:
the skill is available to any agent through MCP
without becoming an entry in the user's repository skill set.

An agent reaches for it when editing `DESIGN.md`,
an equivalent design document,
or design prose that has no repository-owned documentation style to follow.
A repository that ships its own documentation-writing skill or method
should use that instead.

The skill asks the agent to read the target document before editing,
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

The artifact at `.artifacts/design-doc-writer-skill/SKILL.full.md`
is the canonical source for the MCP resource
and stays in sync with `.agents/skills/design-doc-writer/SKILL.md`.
`src/mcp.rs` embeds it as `sirno://skills/design-doc-writer`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
