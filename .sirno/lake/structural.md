---
name: Structural Link
desc: A metadata-backed relation that carries operational Sirno structure.
category:
  - concept
belongs:
  - sirno
  - meta-type
prerequisite:
  - metadata
---

A structural link is an entry-to-entry relation that Sirno reads as project structure.

This repository recommends the `category`, `belongs`, `prerequisite`, and `refines` relations.
An *entry* with `meta.type: "structural"` defines one structural relation.
Its *entry address* is the metadata field name for that relation.
Structural relations are ordinary *entry* metadata fields,
but Sirno treats their values as the graph that powers query, checking,
mist rendering, and tide review worklists.

The relation entry documents that relation's meaning
and carries its Tide policy in `meta.ripple.lake` and `meta.ripple.anchor`.
The marker keeps relation behavior local to the relation entry,
so projection policy can live in mist settings while review policy stays in the lake.
Sirno orders structural relations by entry address wherever an operational order is needed.
Mist settings define generated-footer rendering and rendered relation order.

Structural links refer to *entries* by path.
They are list-valued and may name several targets.
An empty list is still a present field.
Their target order is user-managed.
Sirno preserves target order when parsing, rendering, and moving *entries*.
Humans discover *witness* regions mechanically with `sirno witness ENTRY_ADDRESS --full`.
Agents use the corresponding MCP witness tool.

When `sirno entry rename OLD NEW` renames a relation entry,
it rewrites metadata field names that match `OLD` to `NEW`.
The same operation rewrites structural link target values that name `OLD`.

This *entry* is the review front door for the structural link relation *entries*.
It gives the relation set one review front door while leaving each relation *entry* free
to carry its own meaning and other `belongs` targets.

The *repository witness* for this *entry* should show the generic structural metadata map.
The active relation set is discovered during the raw meta-registry scan
from lake entries with `meta.type: "structural"`.
Rendered directions are defined by the active mist.
