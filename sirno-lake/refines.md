---
name: Refines
desc: A structural link relation from a specific entry to the broader entries it makes concrete.
category:
  - concept
belongs:
  - structural
prerequisite:
  - structural
  - refinement
meta.type: "structural"
meta.ripple.lake: ["to", "from"]
meta.ripple.frost: ["from"]
---

`refines` records a *refinement* edge from a more specific *entry*
to the broader *entries* it makes concrete.

Refinement turns broad design into more specific design,
implementation detail, and testable behavior.
The current *entry* keeps the broader claim attached while making its consequences local.

A refinement chain is a path of increasing specificity.
It starts from a compressed concept and can end near *repository* text.

If the programming language expresses the design most clearly,
the final refinement may be a Markdown code block.

Use `refines` when an *entry* answers the question:
what does this broader design mean here?
The field preserves the reason that a local choice exists.
A low-level *entry* can refine a concept,
a metadata rule can refine the *entry* model,
and a testable behavior can refine a broad invariant.

The more specific *entry* points back to the broader *entry*.
This makes local work easier:
from the local *entry*, a reader can climb back toward intent.
From the broad *entry*, generated or queried metadata can reveal the *entries* that elaborate it.
This entry's `meta.ripple.lake` and `meta.ripple.frost` lists use direct `to` and `from` edges
for *tide* review workitems.
It does not render `refines` generated footer sections by default.
Only waterline `to` participates in tide review.
A `to` target is outgoing metadata on the edited ripple entry itself,
so old broader targets are visible where the edit happens.
Requiring frostline `to` review would turn ordinary retargeting into review noise.
`from` uses both waterline and frostline because incoming refinements live on other entries.
Those dependent entries may not be open during the edit.

Use the nearest broader target that explains the current *entry*'s design pressure.
Do not use `refines` to group *entries* that are merely reviewed together.
Use `belongs` for that horizontal relation.

An *entry* may refine more than one broader *entry*.
That should happen when the local design genuinely joins several ideas,
not when the author wants extra cross-links.
The prose should explain the combined responsibility so a future reader can tell why the field is present.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [structural](structural.md)
- belongs (from): (none)

> **Sirno generated links end.**
