---
name: Narrative Session Discipline
desc: The agent procedure for conducting and materializing a narrative session.
category:
  - meta
belongs:
  - agent-skills
prerequisite:
  - agent-skills
  - portable-agent-skill-language
  - interactive-narrative-session
refines:
  - interactive-narrative-session
---

A narrative session builds an adaptive route through lake knowledge for one reader or task.
Its full MCP resource text lives in `.artifacts/narrative-session-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-narrative-session`.
Its packaged wrapper lives in `.artifacts/narrative-session-discipline/SKILL.md`
and renders to `.agents/skills/sirno-narrative-session/SKILL.md`.

Read the route sources first.
Read `Sirno.toml` for the lake path,
then query or read the active project's narrative, introduction, methodology,
or other route-front-door entries when they exist.
Also read any entries the user named or the task implies.
Do not assume any standard entry address exists.
If a source entry is missing,
state the gap and continue only with the route that can be grounded in existing entries.
Switch to the editor skill if a session discovers a necessary repository,
configuration, witness, or lake maintenance edit.

Maintain a small private session state.
It holds the reader and task,
the design pressure that makes the route useful,
the pull or tension that makes the next concept worth meeting,
known terms and missing prerequisites,
the ordered entry route,
user feedback and corrections,
deferred detail,
the aftertaste phrase or handle,
and the intended narrative entry address when materializing.

Loop in short segments.
Explain the next concept or route choice,
ask for feedback only when the answer changes the next step,
revise the route when the user shows confusion, boredom, urgency, or a sharper goal,
and name what moved earlier, what moved later, and why.
Prefer questions that unlock better sequencing over questions that only feel interactive;
when the next step is clear and the user wants momentum, continue and state the assumption.

Design the route so accurate concepts arrive at the right time.
Show tension before explanation,
give a clean first bite before the full model,
add texture through example, contrast, and consequence,
keep the sequence tight,
honor the reader's agency,
and leave an aftertaste the reader can reuse.

Materialize when the route should guide future onboarding or review.
The artifact is a narrative entry built by the serializer contract; see `narrative-serializer`.
Use prose paragraphs for continuity,
bullets or numbered steps for route structure,
and simple diagrams when they make the path easier for a human reader to inspect.
After materializing, run `sirno_lake_render` and `sirno_lake_check` in edit mode.
Finish by naming the artifact, the entry address, and the main sequencing choice,
and confirm the route preserves pull, a clean first bite, and an aftertaste.
If the serializer script is unavailable or its input contract does not fit the session,
draft the entry manually from the same recorded route state and report that fallback.
If the user wanted only an ephemeral explanation,
do not create a lake entry.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from):
  - [interactive-narrative-session](interactive-narrative-session.md)

> **Sirno generated links end.**
