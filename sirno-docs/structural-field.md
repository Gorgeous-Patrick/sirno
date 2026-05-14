---
name: Structural Field
description: A metadata field that carries operational Sirno structure.
category:
  - concept
belongs:
  - sirno
---

A structural field is a metadata field that Sirno reads as project structure.

The structural fields are `category`, `belongs`, and `refines`.
They are ordinary entry metadata,
but Sirno treats their values as the graph that powers query, checking, and generated links.

Structural fields refer to entries by id.
They are list-valued and may name several targets.
Repository witness status is not a structural field.
Agents should discover witness regions mechanically with `sirno witness ENTRY_ID --full`.

This entry is the review front door for the structural field entries.
It gives the field set one review front door while leaving each field entry free
to carry its own meaning and other `belongs` targets.

The repository witness for this entry should show the field-name constants that define
the structural field set.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from):
- [belongs](belongs.md)
- [category](category.md)
- [refines](refines.md)

Belongs (to):
- [sirno](sirno.md)

> **Sirno generated links end.**
