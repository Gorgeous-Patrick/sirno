---
name: Entry
desc: A named Markdown document in the Sirno Lake.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - sirno-lake
---

An *entry* is a Markdown file in the Sirno Lake.

The filename stem is the local *entry atom*.
The local atom is unique inside its *entry domain*,
case-sensitive, and validated as a cross-platform filename stem.
In a flat *lake*, the root domain is the only domain,
so local atoms are lake-unique.
Write local *entry atoms* as lowercase ASCII kebab-case by default.
That style is easy to type, quote, link, and compare across tools.
It may use spaces, uppercase letters, selected punctuation, and Unicode
when those characters are safe in common filesystems.
It must not use path separators, control characters, Windows-reserved punctuation,
reserved device names, a trailing space, or `.`.
The `.` character separates segments in an *entry address*.
An *entry address* combines *entry domains* with a local atom to find an entry.
It is a lookup form,
not necessarily a unique identity for the entry it resolves to.
Each atom can use at most 252 UTF-8 bytes,
so the final `.md` filename stays inside common component limits.

Each *entry* has a YAML metadata block and a prose body.
The required metadata fields are `name` and `desc`.
This repository recommends `category`, `belongs`, `prerequisite`, and `refines`.
The active structural field set is configured in `Sirno.toml`.
The `meta.frozen` field stores protection reasons such as `reviewed` and `managed`.
`sirno freeze ENTRY_ADDRESS` adds `reviewed`.
Crystallization adds `managed`.
An *entry* file may use LF or CRLF line endings.
Use one line-ending style per file so byte-preserving tools can keep the file predictable.

An *entry* should be focused enough to read in place.
It can state a concept, category, review neighborhood, knowledge dependency,
refinement, invariant, interface, implementation commitment, witnessable claim,
or narrative route.

The body of an *entry* should be useful prose, not just a label.
It should tell a future reader what the *entry* means,
why it deserves a stable name,
and how it participates in the project model.
When the *entry* describes a local implementation commitment,
the body should explain the durable design fact rather than narrating the most recent edit.

The metadata block carries structure that tools must read exactly.
The body carries judgment, examples, and explanation.
This split lets Sirno stay simple.
It can validate addresses and *structural fields*
without pretending to understand the full meaning of the prose.

Good *entries* are compact but not cryptic.
They avoid repeating every repository artifact,
but they also give enough context that a reader can follow a query result without opening ten files.
If a concept depends on several other concepts,
the *structural fields* should carry the navigational structure,
and the prose should explain the local meaning in ordinary language.

When an *entry* has *repository* evidence,
its prose may briefly say what the *witness* is expected to demonstrate.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from):
  - [entry-address-resolution](entry-address-resolution.md)
  - [entry-artifact](entry-artifact.md)
  - [entry-freeze](entry-freeze.md)
  - [entry-lifecycle](entry-lifecycle.md)

> **Sirno generated links end.**
