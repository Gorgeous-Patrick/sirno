---
name: Refines
desc: A structural field from a specific entry to the broader entries it makes concrete.
category:
  - concept
belongs:
  - structural-field
---

`refines` points from a more specific entry to the broader entries it makes concrete.

Refinement turns high-level design into lower-level design,
implementation detail, and testable behavior.
The current entry keeps the broader claim attached while making its consequences local.

A refinement chain is a path of increasing specificity.
It starts from a compressed concept and can end near repository text.

If the programming language expresses the design most clearly,
the final refinement may be a Markdown code block.

Use `refines` when an entry answers the question:
what does this broader design mean here?
The field preserves the reason that a local choice exists.
A low-level entry can refine a concept,
a metadata rule can refine the entry model,
and a testable behavior can refine a broad invariant.

The more specific entry points back to the broader entry.
This makes local work easier:
from the local entry, a reader can climb back toward intent.
From the broad entry, generated or queried metadata can reveal the entries that elaborate it.

Use the nearest broader target that explains the current entry's design pressure.
Do not use `refines` to group entries that are merely reviewed together.
Use `belongs` for that horizontal relation.

An entry may refine more than one broader entry.
That should happen when the local design genuinely joins several ideas,
not when the author wants extra cross-links.
The prose should explain the combined responsibility so a future reader can tell why the field is present.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [structural-field](structural-field.md)
- belongs (from): (none)

> **Sirno generated links end.**
