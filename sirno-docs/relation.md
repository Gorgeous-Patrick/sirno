---
name: Relation
description: A category for entries that define structural entry connections.
category:
  - meta
---

Relations connect entries through metadata.

Sirno treats four relation fields as structural:
`category`, `clustee`, `refiner`, and `witness:`.

The first three fields are lists of entry ids.
The `witness:` marker has no value and uses the entry id as its repository query key.

Operational structure comes only from metadata.
Markdown links in prose may help readers and external tools,
but they do not define Sirno structure.

This keeps the store readable and exact at the same time.
The prose can stay natural,
while the metadata gives tools a small surface they can parse without guessing.
When a relation matters to Sirno,
it belongs in the metadata block.
When a prose link helps the reader,
it can appear in the body without changing the structural graph.

The relations cover different questions.
`category` asks what kind of entry this is.
`clustee` asks which named neighborhood this entry belongs to.
`refiner` asks which broader entry this entry makes more specific.
`witness:` asks whether repository evidence is expected under this entry id.

Those questions should remain distinct.
An entry can be a concept, belong to the Sirno store neighborhood,
refine a broader surface concept, and have repository witnesses.
Combining those roles in one vague tag would make navigation easier at first
and less precise as the store grows.

Relations are ordinary design material.
They should be used when they help a reader or tool find the right context,
not to satisfy a taxonomy for its own sake.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- none
> **Sirno generated links end.**
