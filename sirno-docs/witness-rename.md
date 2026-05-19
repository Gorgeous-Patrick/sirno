---
name: Witness Rename
desc: Rewriting only the captured entry id when sirno entry rename changes an id.
category:
  - concept
belongs:
  - sirno-witness
---

*Witness rename* keeps *repository witness* blocks pointing at the right *entry*
when `sirno entry rename` changes an *entry* id.

Sirno scans the configured repo members the same way *witness lookup* does.
For each block whose captured id is the old id,
it builds text edits for the first capture group
of the opening and closing delimiters,
then applies them in place.

Only the captured id is rewritten.
The sentinel syntax around it is left alone,
and the block body stays owned by the repository artifact.
A rename where the old and new ids are equal,
or a project with no configured repo surface,
changes nothing.

This makes an *entry* id a safe thing to change.
The id is the single link between a claim and its evidence,
so renaming the claim updates exactly that link
without touching the witnessed code.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-witness](sirno-witness.md)
- belongs (from): (none)

> **Sirno generated links end.**
