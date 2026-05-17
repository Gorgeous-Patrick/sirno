---
name: Sirno Lake
desc: The configured directory of compact named design entries.
category:
  - concept
belongs:
  - form
---

The Sirno Lake is the configured directory of Markdown *entries*.
The usual convention is `docs/`.

The name keeps Sirno's Cirno influence visible.
It echoes Misty Lake and recasts the project design source as a Misty Lake of Knowledge:
quiet enough to read,
but structured enough that changes leave visible *ripples*.

`Sirno.toml` records the *lake* path under `[lake].path`.
`sirno move PATH` renames the configured *lake* directory
and writes the new path back to `[lake].path`.
`sirno mv PATH` is its short form.
The move refuses to replace an existing destination.

The *lake* is the human-readable intermediate representation:
text first, structured enough for tools,
and compact enough for humans and agents to inspect locally.
When Sirno Frost is configured,
its frozen state is versioned through a separate `eter` *frost* path,
so one version names one immutable set of *entries*.

Each *entry* is an ordinary Markdown file with a YAML metadata block and prose body.
The filename stem is the stable id used by *structural fields*, *generated footers*, and *witness* lookup.
The id is filename-like by definition.
Lowercase kebab-case is a convention for readable *lakes*, not a validation boundary.

Once established, the *lake* is the preferred structured design source.

The *lake* should feel like a set of well-named design cards.
Each card has enough prose to be useful on its own,
but it also participates in a larger graph through metadata.
The graph is intentionally small:
classification, belonging, refinement, and *witnesses*.
That small set is enough to navigate without turning the *lake* into a separate database language.
The `frozen:` marker adds a file-level protection state,
so one current Frost-backed public *entry* can be held read-only
and checked against the Frost snapshot before commit.

The *lake* is also a collaboration boundary.
A person can edit an *entry* directly.
A CLI can check its metadata and links.
An agent can query a few related *entries* before changing code.
An editor can use *generated footers* to expose navigation.
All of those forms use the same filenames and metadata.

The *lake* is a working form.
Direct edits become frozen versions only when Sirno Frost is configured
and Sirno freezes the *lake* into the *frost* path.

Some files under a *lake* root may belong to adjacent tools.
`[lake].ignore` lists paths relative to the *lake* root.
An ignored path covers the path itself and its descendants.
This lets a *lake* contain editor state such as `.obsidian`
without making that state part of the Sirno *entry* set.

The *lake* should avoid becoming either a glossary or a backlog.
A glossary *entry* may define a word without carrying design pressure.
A backlog item may describe work without preserving the concept behind it.
An *entry* should name durable project knowledge:
why a commitment exists,
how it connects to broader design,
and where implementation evidence should be found when that evidence exists.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [form](form.md)
- belongs (from):
  - [design-source-authority](design-source-authority.md)
  - [entry](entry.md)
  - [entry-lifecycle](entry-lifecycle.md)
  - [metadata](metadata.md)
  - [project-config](project-config.md)
  - [query](query.md)
  - [refinement](refinement.md)
  - [ripple](ripple.md)
  - [structural-check](structural-check.md)

> **Sirno generated links end.**
