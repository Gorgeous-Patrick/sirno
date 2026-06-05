---
name: Metadata
desc: The exact YAML schema that carries Sirno entry structure.
category:
  - concept
belongs:
  - entry
prerequisite:
  - entry
---

Metadata is the exact schema that carries Sirno structure.

Every *entry* has a YAML metadata block.
Metadata belongs to `entry` because it describes the schema surface of each entry.

| Field | Shape | Meaning |
|---|---|---|
| discovered intrinsic field | plain string | Required field defined by an intrinsic entry. |
| `meta` | mapping | Optional Sirno-managed metadata. |
| `meta.frozen` | non-empty reason list | Declares that the lake *entry* is protected. |
| `meta.type: "intrinsic"` | scalar marker | Marks an entry as an intrinsic metadata field. |
| `meta.type: "structural"` | scalar marker | Marks a structural relation definition. |
| `meta.ripple.lake` | direction list | Defines how waterline *tide* follows a structural relation. |
| `meta.ripple.anchor` | direction list | Defines how Anchor-side *tide* follows a structural relation. |

Sirno resolves metadata in two phases.
The first phase scans raw entry frontmatter for `meta.type`.
It writes the generated `meta-registry` lockfile.
The second phase uses the entry's ownership-scope registry
to parse intrinsic fields and structural relations.

The `meta-type` entry groups the `meta.type` discriminator values:

| Value | Role entry | Valid carriers |
|---|---|---|
| `intrinsic` | `intrinsic` | Entries that define required plain-string metadata fields. |
| `structural` | `structural` | Structural relation entries. |

The current lake defines `name` and `desc` as intrinsic fields.
Structural relation entries belong to `structural`.

Frozen reasons are:

| Reason | Meaning |
|---|---|
| `reviewed` | Deprecated manual protection reason. |
| `managed` | Crystallization owns the entry content. |

The `reviewed` reason belongs to the deprecated manual freeze design.
An entry may carry both frozen reasons while the field exists.

Ripple fields are present only on entries that define structural link relations.
Their `to`, `from`, and `clique` values enable waterline or Anchor-side review workitems.
Empty `meta.ripple.lake` and `meta.ripple.anchor` lists mean the relation has no tide behavior.

Structural link relations are optional.
This repository defines `category`, `belongs`, `prerequisite`, and `refines`.
They follow three rules:

- They are always lists when present.
- Their values are *entry addresses*.
- An empty list is a present empty field.

Structural relation order is entry-address order.
Rendered relation order belongs to mist settings.
Structural target order stays user-authored metadata.
Sirno preserves target order when parsing, rendering, and moving *entries*.

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
the structural link metadata must say so.
If a tool needs to know that an entry defines an intrinsic metadata field,
the entry must carry `meta.type: "intrinsic"`.
If a tool needs typed intrinsic or structural fields,
it must use the registry discovered for the entry's ownership scope.
If an agent needs to inspect *repository* evidence for an *entry*,
it should use the agent-facing MCP tool.
If a human needs the same evidence,
run `sirno witness ENTRY_ADDRESS --full`.

A canonical *entry* shape looks like this:

```yaml
---
name: Category
desc: A structural link relation that classifies an entry by other entries.
meta.type: "structural"
meta.ripple.lake: []
meta.ripple.anchor: []
category:
  - category
---
```

The schema keeps required scalar fields small.
New list-valued metadata can become a structural link relation
when an entry with the same address declares `meta.type: "structural"`.
List-valued metadata fields without a matching relation entry remain visible as check warnings.
