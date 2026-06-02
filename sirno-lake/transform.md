---
name: Transform
desc: The concept and review neighborhood for movement between Sirno forms.
category:
  - concept
belongs:
  - form
prerequisite:
  - sirno-lake
  - repo
---

A *transform* names intentional movement between Sirno *forms*.
It gives a direction, a source form, a target form, and a review habit.
A *transform* may be a command, a skill workflow, a manual editing practice, or a narrative route.

The *transform* names make design work easier to request and review.
Instead of saying "update the code, README, or generated documents from the lake",
a user can ask to actualize the *lake* into the *repository*.
Instead of saying "move this repository fact into the canonical design source",
a user can ask to internalize the *repository* into the *lake*.

The polarity is simple.
The *lake* is canonical.
The *repository* is material:
source code, tests, configuration, generated files, README files,
design documents outside the *lake*, and other inspectable artifacts.
Actualization gives those materials shape from the *lake*.
Internalization lets those materials teach the *lake* what durable design knowledge they expose.

Current vocabulary:

- `actualize`, also named `lake-to-repo`, moves from *lake* design to *repository* material.
- `internalize`, also named `repo-to-lake`, moves from *repository* material to durable *lake* design.

This vocabulary names the current directions, not the limit of the concept.
A future *transform* should define its own entry, direction, source form, target form, and evidence habits.
It should change this entry only when the broader transform contract changes.

A reviewer who touches one direction should visit the other direction through this entry.
The round trip matters because repo material may reveal that the *lake* needs sharper names,
and a sharper *lake* should make later repo material easier to regenerate.

This vocabulary also helps skills stay focused.
An actualization skill should inspect *entries* before editing repository material.
An internalization skill should turn repository material into durable *entries*,
metadata, artifacts, or witness expectations.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [form](form.md)
- belongs (from):
  - [actualize](actualize.md)
  - [internalize](internalize.md)

> **Sirno generated links end.**
