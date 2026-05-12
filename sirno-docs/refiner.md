---
name: Refiner
description: A structural field from a specific entry to the broader entries it refines.
category:
  - concept
---

`refiner` points from a more specific entry to the broader entries it refines.

Refinement turns high-level design into lower-level design,
implementation detail, and testable behavior.
The refined entry keeps the concept intact while making its consequences local and concrete.

A refinement chain is a path of increasing specificity.
It starts from a compressed concept and can end near repository text.

If the programming language expresses the design most clearly,
the final refinement may be a Markdown code block.

Use `refiner` when an entry answers the question:
what does this broader design mean here?
The field preserves the reason that a local choice exists.
A low-level entry can refine a concept,
a metadata rule can refine the entry model,
and a testable behavior can refine a broad invariant.

Refinement is directional.
The more specific entry points back to the broader entry.
That direction makes local work easier:
from the local entry, a reader can climb back toward intent.
From the broad entry, generated or queried metadata can reveal the entries that elaborate it.

An entry may refine more than one broader entry.
That should happen when the local design genuinely joins several ideas,
not when the author wants extra cross-links.
The prose should explain the combined responsibility so a future reader can tell why the field is present.

---

> **Sirno generated links begin. Do not edit this section.**

- none

> **Sirno generated links end.**
