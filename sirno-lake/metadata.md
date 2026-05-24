---
name: Metadata
desc: The exact YAML schema that carries Sirno entry structure.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry
---

Metadata is the exact schema that carries Sirno structure.

Every *entry* has a YAML metadata block.
The required fields are `name` and `desc`,
both plain strings.

`meta` is optional Sirno-managed metadata.
It is a mapping.
`meta.frozen` declares that the lake *entry* is protected.
It is a non-empty list of protection reasons when present.
`reviewed` means the entry matches the current frost snapshot.
The frost layer accepts reviewed entries only while their committed form still matches that snapshot.
`managed` means crystallization owns the entry content.
An entry may carry both reasons.
Flat `meta.lake.*` and `meta.frost.*` fields define how *tide* follows a structural relation.
They are present on entries that define configured structural link relations.
Their `to`, `from`, and `clique` booleans enable waterline or frostline review workitems.
`meta.lake: false` and `meta.frost: false` mean the relation has no tide behavior.

Configured structural link relations are optional.
This repository configures `category`, `belongs`, `prerequisite`, and `refines`.
They are always lists when present,
and their values are *entry addresses*.
An empty list is a present empty field.
Their relation order is user-authored metadata.
Sirno preserves it when parsing, rendering, and moving *entries* through Sirno Frost.

Operational structure is formed only from metadata.
Prose links may help readers and external tools,
but they do not define Sirno structure.

The metadata block should be small and stable.
It is the part of an *entry* that tools must read without interpretation.
That is why required fields are plain strings,
and structural links are lists of entry addresses.

The body can explain nuance,
but the metadata must not require prose parsing.
If a tool needs to know that one *entry* depends on or refines another,
the configured structural link metadata must say so.
If an agent needs to inspect *repository* evidence for an *entry*,
it should use the agent-facing MCP tool.
If a human needs the same evidence,
run `sirno witness ENTRY_ADDRESS --full`.

A canonical *entry* shape looks like this:

```yaml
---
name: Category
desc: A structural link relation that classifies an entry by other entries.
meta.lake: false
meta.frost: false
category:
  - category
---
```

The schema keeps required scalar fields small.
New list-valued metadata can become a structural link relation
when `[structural.FIELD]` configures that field.
Unconfigured list-valued metadata fields remain visible as check warnings.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
