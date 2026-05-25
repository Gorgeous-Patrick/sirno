---
name: Generated Footer
desc: A Sirno-owned footer that projects selected metadata fields as links.
category:
  - concept
belongs:
  - generated-navigation
prerequisite:
  - structural
---

Sirno may generate and maintain a footer at the bottom of *entries*.

The *generated footer* *entry* is the front door for generated navigation regions.
Its local refinements define ownership boundaries and link selection policy.

The footer is bounded by sentinels that state Sirno owns the region.
Humans and tools should leave that region untouched.

The sentinels are human-visible Markdown block quotes.
The generated list is separated from both sentinels by blank lines.
That shape keeps Markdown renderers from nesting the closing sentinel under the list.

The schematic shape is:

```text
[optional horizontal divider]
> begin generated footer marker

- relation (edge):
  - linked entry
- relation (edge): (none)

> end generated footer marker
```

When Sirno appends a *generated footer* to a non-empty body,
it inserts a horizontal divider immediately before the generated region
unless the body already ends with one.

The footer projects metadata-derived structure for external tools that navigate links.
It is not the source of structural truth.

The *generated footer* is an interoperability layer.
Some editors and documentation tools navigate Markdown links more naturally than metadata fields.
Sirno can project selected fields into links so those tools can participate in the *lake*.

The generated body is grouped by configured link relation.
Each direction listed under `[render.structural]` appears in the region.
Within one relation,
groups render in `to`, `from`, then `clique` order.
Each group is a top-level Markdown list item,
such as `- category (from):`, `- belongs (to):`, or `- belongs (clique):`.
The group's links are child list items indented under the group item.
A group with no links is rendered inline, such as `- belongs (from): (none)`.
If no rendered group is enabled, the region contains `(none)`.

The footer is derived from metadata.
Changing a generated link by hand does not change the metadata.
Changing the metadata and regenerating the footer is the correct path.
The sentinels make that ownership boundary visible in the *entry* file itself.
A *frost* commit removes the *generated footer* before writing the *entry* snapshot.
The frost layer keeps canonical metadata and prose,
not navigation projections.

The `[render.structural]` policy controls which link relations appear.

`sirno check` reports stale *generated footer* regions when render checking is enabled.
`sirno render` creates or replaces *generated footer* regions.
`sirno render --dry` reports *generated footer* regions that would change without writing files.
`sirno render --override-json JSON` applies temporary `[render.structural]` settings.
The override replaces the configured settings in memory and does not update `Sirno.toml`.
`sirno render delete` removes them.
The mutating commands leave prose outside the guard-bounded region under user ownership.
`sirno rg` searches the *lake* as if those guard-bounded regions contain only whitespace.
That keeps literal searches focused on authored metadata and prose.
`sirno rg --with-generated-footer` includes the projected links when they are the search target.

The *generated footers* should stay boring.
Their job is to make the edges of the page useful to tools,
not to become another place for design prose.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation](generated-navigation.md)
- belongs (from): (none)

> **Sirno generated links end.**
