---
name: Semantic Locality
desc: A principle for writing entries whose meaning stays local.
category:
  - meta
  - concept
belongs:
  - methodology
prerequisite:
  - methodology
  - entry
  - refinement
refines:
  - methodology
---

Semantic locality is the entry-writing rule that each *entry* should remain meaningful when read alone.
A reader should understand the design object before following generated links, witnesses,
or neighboring refinements.

Semantic locality still allows useful lists.
A narrative route, index, roster, or review front door may enumerate children
when that list is part of the entry's local claim.
The prose should say what the list is for,
so a reader can tell the difference between the entry's semantic contract
and its current route through narrower entries.

An *entry* owns a local claim: a term, constraint, behavior, method, interface,
or implementation commitment.
Metadata carries navigation.
Repository *witnesses* carry evidence.
Refinement entries carry local specialization.
The body may mention nearby concepts when a sentence needs them,
but it should not require unstated neighbors to make basic sense.

A broad *entry* should describe the semantic commitment shared by possible refinements.
It should leave current implementation entries, command surfaces, repository modules,
and concrete witnesses to narrower entries.
A new implementation should be addable by creating or revising its own local *entry*,
structural links, and witnesses,
without rewriting the general *entry*'s prose.

Pointers still matter.
Use structural metadata for links that improve review, navigation, or accountability:

- Use `prerequisite` when another concept unlocks the current claim.
- Use `refines` when a local *entry* narrows a broader design claim.
- Use `belongs` when related *entries* should be reviewed together.
- Use a *witness* when repository material demonstrates the exact claim.

The body should make a pointer helpful rather than mandatory.
If a linked *entry*, upstream lake, repository *witness*, or generated footer is unavailable,
the *entry* should still say what it names and why it matters.
Missing surroundings may reduce evidence or context,
but they should not erase the local meaning.
Changing that list may be appropriate when the route changes.
Changing a broad semantic contract should not be required merely because a new child,
implementation, or witness appears.

A practical check:

- Can the *entry* be understood from its own metadata and body?
- Does it name only the commitment it owns?
- Are implementation details kept in entries that refine or witness it?
- Is any child list presented as a route, roster, index, or review front door?
- Can a new implementation be added without rewriting the broad semantic contract?
- Are necessary pointers represented as structural links instead of prose inventories?

For example,
a general transform *entry* should state the transform's source form, target form,
and durable semantic contract.
A command entry, interface entry, or repository module entry can refine or witness that contract locally.
The transform *entry* remains open to later implementations because it owns the meaning,
not the inventory of ways to realize it.
