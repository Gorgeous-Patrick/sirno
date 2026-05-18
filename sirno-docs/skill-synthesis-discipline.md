---
name: Skill Synthesis Discipline
desc: The agent procedure for rebuilding packaged agent skills from meta-categorized lake entries.
category:
  - meta
belongs:
  - agent-skills
---

Skill synthesis rebuilds the packaged Sirno skills from the lake's `meta`-categorized entries.
It renders to `.agents/skills/sirno-skill-synthesizer/SKILL.md`.

Read the sources first.
Read `Sirno.toml` for the lake path,
then `agent-skills` for the skill roster and the handoffs between skills,
then every `meta`-categorized entry through `sirno query`.
The lake is authoritative; a packaged skill is its reproducible surface.

Separate disciplines from shared method.
A `meta` entry that `belongs: agent-skills` and defines a skill procedure is a skill source.
The other `meta` entries carry vocabulary, principles, perspective, and design authority.
They are cross-cutting method that every skill must respect,
not skills in their own right.
Adjacent documentation-writing skills, such as `design-doc-writer`,
are also method inputs when Sirno skill work touches design prose.
Their reusable content is reader evaluation,
conceptual ordering,
declarative precision,
and whole-document coherence.
They do not become package sources unless the Sirno skill roster adds them.

Map each discipline to one package.
A skill discipline renders exactly one `.agents/skills/sirno-<role>/SKILL.md` package.
The target package path is written in the discipline body until the project defines
a structural field for skill packages.
Keep the existing skill directory name and do not invent a new role
unless `agent-skills` adds one to the roster.
Every `belongs: agent-skills` discipline should have a package,
and every package should trace back to a discipline.

Render, do not reinterpret.
A packaged skill operationalizes its discipline plus the shared `meta` method it depends on.
Frontmatter `name` is the skill directory id;
`description` states when to use the skill and the triggers that should invoke it.
The body turns durable procedure into concrete steps and current commands.
Add nothing the lake does not commit, and drop nothing the discipline requires.
Include the discipline's failure paths in the rendered skill.
These paths cover missing sources, unavailable commands, blocked validation,
absent evidence, and design changes that must reflect back into the lake.

Inspect the current Sirno CLI before writing commands into a skill.
A skill that names a missing command is worse than one that only names the procedure.

Keep the lake the source of truth.
When a packaged skill and the lake disagree,
correct the skill, never the lake.
This discipline is itself a skill source;
the synthesizer rebuilds its own package the same way it rebuilds the others.

Validate after writing.
Run render maintenance if lake metadata changed,
then the review-mode structural check.
Confirm each SKILL.md has valid frontmatter,
and that the disciplines and packages still correspond one to one.
If a package exists without a discipline,
either add the missing discipline to the lake or report the package as outside the reproducible set.
If a discipline exists without a package,
create the package only when the roster says the skill should ship.
If a skill's generated procedure would need behavior the lake does not commit,
leave that behavior out and report the missing design instead of inventing it.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
