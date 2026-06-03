---
name: Tide Workitem
desc: One dependency review obligation in a Sirno Tide.
category:
  - concept
  - implemented
belongs:
  - tide
prerequisite:
  - tide
---

A *tide workitem* is one dependency review obligation.
Its identity is the tuple `(ripple, field, direction, neighbor)`.
Sirno does not create a separate workitem id.

`ripple` names the changed *entry* that created the obligation.
`field` names the configured link relation.
`direction` is `to`, `from`, or `clique`.
`neighbor` names the *entry* that must be reviewed.

The same tuple can be produced through the waterline, Anchor, or both.
Full status output shows one workitem with its source list,
such as `lake`, `anchor`, or both.
The source list explains why the obligation exists without changing its identity.

Text commands may name a full workitem as comma-separated fields.
Entry addresses and link relation names cannot contain commas,
so `ripple,field,direction,neighbor` is unambiguous.
JSON input can carry the same tuple when a caller needs structured command input.

Resolving by neighbor id resolves open workitems whose `neighbor` is that *entry*.
That means the reviewer inspected the neighbor and accepts its current state.
Resolving a full tuple records only that exact obligation.
