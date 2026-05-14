---
name: Generated Footer
description: A Sirno-owned footer that projects selected metadata fields as links.
category:
  - concept
belongs:
  - sirno-lake
---

Sirno may generate and maintain a footer at the bottom of entries.

The generated footer entry is the front door for generated navigation regions.
Its local refinements define ownership boundaries and link selection policy.

The footer is bounded by sentinels that state Sirno owns the region.
Humans and tools should leave that region untouched.

The sentinels are human-visible Markdown block quotes.
The generated list is separated from both sentinels by blank lines.
That shape keeps Markdown renderers from nesting the closing sentinel under the list.

When Sirno appends a generated footer to a non-empty body,
it inserts a horizontal divider immediately before the generated region
unless the body already ends with one.

The footer projects metadata-derived structure for external tools that navigate links.
It is not the source of structural truth.

The generated footer is an interoperability layer.
Some editors and documentation tools navigate Markdown links more naturally than metadata fields.
Sirno can project selected fields into links so those tools can participate in the lake.

The generated body is grouped by configured structural field.
Each enabled group appears in the region.
A group with links begins with a colon-terminated label,
such as `Category (from):`, `Belongs (to):`, or `Clique:`.
The group's links are ordinary Markdown list items.
A group with no links is rendered inline, such as `Belongs (from): (none)`.
If no generated-link group is enabled, the region contains `(none)`.

The footer is derived from metadata.
Changing a generated link by hand does not change the metadata.
Changing the metadata and regenerating the footer is the correct path.
The sentinels make that ownership boundary visible in the entry file itself.
Frost commits remove the generated footer before writing the entry snapshot.
Sirno Frost keeps canonical metadata and prose,
not navigation projections.

The configured link policy controls which structural fields appear.

`sirno check` reports stale generated footer regions when link checking is enabled.
`sirno gen-link` creates or replaces generated footer regions.
`sirno gen-link --dry` reports generated footer regions that would change without writing files.
`sirno gen-link delete` removes them.
The mutating commands leave prose outside the guard-bounded region under user ownership.

Generated footers should stay boring.
Their job is to make the edges of the page useful to tools,
not to become another place for design prose.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to):
- [sirno-lake](sirno-lake.md)

> **Sirno generated links end.**
