---
name: Generated Footer
description: A Sirno-owned footer that projects metadata relations as links.
category:
  - concept
clustee:
  - sirno-store
---

Sirno may generate and maintain a footer at the bottom of entries.

The footer is bounded by sentinels that state Sirno owns the region.
Humans and tools should leave that region untouched.

The footer format is configurable.
It can use ordinary Markdown links or Obsidian-style links.

The footer projects metadata-derived structure for external tools that navigate links.
It is not the source of structural truth.

The generated footer is an interoperability layer.
Some editors and documentation tools navigate Markdown links more naturally than metadata fields.
Sirno can project selected relations into links so those tools can participate in the store.

The footer is derived from metadata.
Changing a generated link by hand does not change the relation.
Changing the metadata and regenerating the footer is the correct path.
The sentinels make that ownership boundary visible in the entry file itself.

The configured link policy controls which relation fields appear.
In this repository, clustee links are generated,
while category and refiner links remain metadata-only.
That choice keeps the visible footer focused on neighborhood navigation.

Generated footers should stay boring.
Their job is to make the edges of the page useful to tools,
not to become another place for design prose.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [sirno-store](sirno-store.md)
> **Sirno generated links end.**
