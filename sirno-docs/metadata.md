---
name: Metadata
desc: The exact YAML schema that carries Sirno entry structure.
category:
  - concept
belongs:
  - lake
prerequisite:
  - entry
---

Metadata is the exact schema that carries Sirno structure.

Every *entry* has a YAML metadata block.
The required fields are `name` and `desc`,
both plain strings.

Configured structural fields are optional.
This repository configures `category`, `belongs`, `prerequisite`, and `refines`.
They are always lists when present,
and their values are *entry* ids.
An empty list is a present empty field.
Their field order is user-authored metadata.
Sirno preserves it when parsing, rendering, and moving *entries* through Sirno Frost.

`frozen:` declares that the public *entry* is protected
because it matches the current Sirno Frost snapshot.
Sirno Frost accepts it only while its committed form still matches that snapshot.
It is written without a value.

Operational structure is formed only from metadata.
Prose links may help readers and external tools,
but they do not define Sirno structure.

The metadata block should be small and stable.
It is the part of an *entry* that tools must read without interpretation.
That is why required fields are plain strings,
and structural fields are lists of ids.

The body can explain nuance,
but the metadata must not require prose parsing.
If a tool needs to know that one *entry* depends on or refines another,
the configured structural metadata must say so.
If an agent needs to inspect *repository* evidence for an *entry*,
it should use the agent-facing MCP tool.
If a human needs the same evidence,
run `sirno witness ENTRY_ID --full`.

A canonical *entry* shape looks like this:

```yaml
---
name: Concept
desc: A named idea that compresses project knowledge.
category:
  - concept
---
```

The schema keeps required scalar fields small.
New list-valued metadata can become structural when `[structural.FIELD]` configures that field.
Unconfigured list-valued metadata fields remain visible as check warnings.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [lake](lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
