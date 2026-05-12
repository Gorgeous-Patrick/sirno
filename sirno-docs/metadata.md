---
name: Metadata
description: The exact YAML schema that carries Sirno entry structure.
category:
  - concept
clustee:
  - sirno-store
---

Metadata is the exact schema that carries Sirno structure.

Every entry has a YAML metadata block.
The required fields are `name` and `description`,
both plain strings.

The optional structural fields are `category`, `clustee`, and `refiner`.
They are always lists when present, and their values are entry ids.

The optional `witness:` marker is canonical and has no value.
No other witness spelling is accepted.

Operational structure is formed only from metadata.
Prose links may help readers and external tools,
but they do not define Sirno structure.

The metadata block should be small and stable.
It is the part of an entry that tools must read without interpretation.
That is why required fields are plain strings,
structural fields are lists of ids,
and the witness marker has no value.

The body can explain nuance,
but the metadata must not require prose parsing.
If a tool needs to know that one entry refines another,
the `refiner` field must say so.
If a tool needs to know whether an entry has repository evidence,
the `witness:` marker must be present.

A canonical entry shape looks like this:

```yaml
---
name: Witness
description: An entry whose claim is evidenced by repository artifacts.
category:
  - concept
witness:
---
```

The schema is intentionally conservative.
Adding fields is future design work,
because every accepted field becomes part of the public structure that readers and tools may rely on.

---

> **Sirno generated links begin. Do not edit this section.**

- [sirno-store](sirno-store.md)

> **Sirno generated links end.**
