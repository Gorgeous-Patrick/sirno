---
name: Agent Skills
desc: The Sirno agent skill set and the handoffs between its skills.
category:
  - meta
belongs:
  - sirno
---

Sirno ships an agent skill set that renders this lake's method into operational procedure.

There are five Sirno skills.
The editor skill creates, revises, and reorganizes entries,
and moves design knowledge from a monograph into the lake.
The explorer skill reads a Sirno-managed repository from the lake outward to locate design
and its evidence.
The witness skill links lake entries to repository evidence and keeps that evidence precise.
The narrative-session skill conducts an adaptive route through lake knowledge
and materializes it as a narrative entry when the route should persist.
The skill-synthesizer skill rebuilds the packaged Sirno skills from discipline entries,
so the skill set stays a reproducible surface of the method rather than prose that can drift.

This entry is the review front door for those skills.
The durable procedure each skill encodes lives in its own discipline entry,
so a skill can be rebuilt from the lake rather than only from its packaged prose.
Each discipline entry names its target `.agents/skills/sirno-*/SKILL.md` package path.
The discipline entries are `lake-editing-discipline`, `lake-exploration-discipline`,
`witness-linking-discipline`, `narrative-session-discipline`,
and `skill-synthesis-discipline`.

The skills hand off rather than overlap.
Exploration switches to the witness skill when the task changes from reading evidence
to creating or refining it.
Exploration switches to the editor skill when the task changes from reading entries to editing them.
The editor skill defers to the repository documentation-writing skills for `README`, `DESIGN`,
and `METHODOLOGY` prose, because those documents have their own roles and style.
The `design-doc-writer` skill is the adjacent meta-management skill for design documents.
Here, meta-management means maintaining design documents about the project and its method.
It is not part of the five packaged Sirno skills.
The `design-doc-writer` skill contributes reusable design-document habits:
read the whole design document,
order sections by conceptual dependency and scope,
write declarative, dry, precise prose,
prefer positive definitions over defensive framing,
and evaluate the result as a reader before and after editing.
Sirno skill work uses those habits whenever it touches design documents or design entries.
The synthesis skill checks the Sirno skill roster and reports any discipline or package
that no longer has a counterpart.

A skill is an operational rendering of lake method, not a separate authority.
When a skill and the lake disagree, the lake and `Sirno.toml` win,
and the skill should be corrected.
Failure handling is part of the rendering.
A packaged Sirno skill should say what to do when sources are missing,
checks fail, commands are unavailable, or the requested evidence does not exist.
This keeps the lake the source of truth and the skills its reproducible surface.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [lake-editing-discipline](lake-editing-discipline.md)
  - [lake-exploration-discipline](lake-exploration-discipline.md)
  - [narrative-session-discipline](narrative-session-discipline.md)
  - [skill-synthesis-discipline](skill-synthesis-discipline.md)
  - [witness-linking-discipline](witness-linking-discipline.md)

> **Sirno generated links end.**
