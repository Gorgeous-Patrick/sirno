---
name: Sirno Store
description: The configured directory of compact named design entries.
category:
  - concept
clustee:
  - sirno
refiner:
  - surface
---

The Sirno store is the configured directory of Markdown entries.
The usual convention is `docs/`.

The configured project in this repository uses `sirno-docs/`.
`Sirno.toml` records that path under `[store].path`.

The store is the human-readable intermediate representation:
text first, structured enough for tools,
and compact enough for humans and agents to inspect locally.

Each entry is an ordinary Markdown file with a YAML metadata block and prose body.
The filename stem is the stable id used by structural fields, generated footers, and witness lookup.

Once established, the store is the preferred structured design source.

The store should feel like a set of well-named design cards.
Each card has enough prose to be useful on its own,
but it also participates in a larger graph through metadata.
The graph is intentionally small:
classification, clique membership, refinement, and witnesses.
That small set is enough to navigate without turning the store into a separate database language.

The store is also a collaboration boundary.
A person can edit an entry directly.
A CLI can check its metadata and links.
An agent can query a few related entries before changing code.
An editor can use generated footers to expose navigation.
All of those surfaces use the same filenames and metadata.

Some files under a store root may belong to adjacent tools.
`[store].ignore` lists store-root-relative paths that Sirno skips.
An ignored path covers the path itself and its descendants.
This lets a store contain editor state such as `.obsidian`
without making that state part of the Sirno entry set.

The store should avoid becoming either a glossary or a backlog.
A glossary entry may define a word without carrying design pressure.
A backlog item may describe work without preserving the concept behind it.
A Sirno entry should name durable project knowledge:
why a commitment exists,
how it connects to broader design,
and where implementation evidence should be found when that evidence exists.

---

> **Sirno generated links begin. Do not edit this section.**

Category (from): (none)

Category (to)
- [concept](concept.md)

Clique
- [code-surface](code-surface.md)
- [concept-driven-development](concept-driven-development.md)
- [entry](entry.md)
- [future-work](future-work.md)
- [generated-footer](generated-footer.md)
- [metadata](metadata.md)
- [mono](mono.md)
- [planning](planning.md)
- [project-config](project-config.md)
- [project-self-application](project-self-application.md)
- [query](query.md)
- [sirno](sirno.md)
- [storage-and-interfaces](storage-and-interfaces.md)
- [structural-check](structural-check.md)
- [surface](surface.md)

> **Sirno generated links end.**
