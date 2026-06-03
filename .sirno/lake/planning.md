---
name: Planning
desc: A use of Sirno entries for durable work artifacts.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - entry
---

Planning lives in skills built on top of Sirno.

The *entries* are durable, named, and structured by metadata.
That structure can support persistent planning without adding a planning primitive to Sirno.

A skill may represent a worklist as ordinary *entries*.
Those *entries* can use categories, `belongs`, `prerequisite`, `refines`, and *witnesses*
like any other *lake entries*.

This keeps the core model small.
Planning often needs state, priorities, sequencing, ownership, and progress signals.
Those concerns vary by team and project.
Sirno provides names, prose, structural links, checks, and *witnesses*;
a planning skill can decide how to express a worklist using those primitives.

The benefit is continuity.
A plan written as Sirno *entries* can refer to the same concepts and implementation commitments as the design *lake*.
It can place related tasks in a review neighborhood,
refine broader design *entries*,
or mark work that should be witnessed by *repository* artifacts.
The plan remains inspectable as Markdown rather than being hidden in a separate task system.

Planning *entries* should still respect the *lake*.
They should not smuggle in new link relations unless the project explicitly designs them.
If a worklist needs special behavior,
that behavior belongs in the skill or in future Sirno design,
not in ad hoc metadata that core tools silently ignore.
