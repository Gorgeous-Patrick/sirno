---
name: Sirno Lake
desc: The configured directory of compact named design entries.
category:
  - concept
belongs:
  - form
prerequisite:
  - form
---

The Sirno Lake is the configured directory of Markdown *entries*.
The usual convention is `docs/`.

The name keeps Sirno's Cirno influence visible.
It echoes Misty Lake and recasts the project design source as a Misty Lake of Knowledge:
quiet enough to read,
but structured enough that changes leave visible *ripples*.

`Sirno.toml` records the *lake* path under `[lake].path`.
`sirno lake move PATH` renames the configured *lake* directory
and writes the new path back to `[lake].path`.
`sirno lake mv PATH` is its short form.
`sirno move lake PATH` and `sirno mv lake PATH` select the same path move.
The move creates missing destination parents and refuses to replace an existing destination.
When `PATH` is inside the current *lake* path,
Sirno stages the directory through a temporary sibling
and recreates the parent path before placing the moved *lake* at `PATH`.

The *lake* is the human-readable intermediate representation:
text first, structured enough for tools,
and compact enough for humans and agents to inspect locally.
When frost is configured,
its frozen state is versioned through a separate `eter` *frost* path,
so one version names one immutable set of *entries*.

Each *entry* is an ordinary Markdown file with a YAML metadata block and prose body.
The filename stem is the local id used by *structural fields*, *generated footers*, and *witness* lookup.
The id is filename-like by definition.
Lowercase kebab-case is a convention for readable *lakes*, not a validation boundary.
The `.` character separates *entry path* segments.
An *entry path* joins *entry domains* and a local id into a lookup form.
Domain segments map to folders in the *lake*.
Several paths may resolve to the same *entry* in a composed lake.
The leading-dot path form `.<id>` is reserved for Sirno built-in functionality.
Project entries and dependency domains use ordinary `<id>` path segments.

A *lake sheaf* is the resolved composition of multiple lakes.
It flattens sublakes to the top level before dependency domains link back
to the resolved top-level entries.
That model resolves diamond dependencies without making dependency versioning part
of the entry naming syntax.

The `.artifacts` directory is reserved for lake-owned *entry artifacts*.
It is a built-in `.<id>` path, not a project-defined *entry domain*.
Artifacts live under `.artifacts/<entry-id>/...`.
This keeps the Sirno Lake *entry* files flat while letting an *entry* own non-Markdown bytes.
The owner directory must be an existing *entry* id.
Artifact paths below that owner are relative UTF-8 paths with only normal components.

Once established, the *lake* is the preferred structured design source.

The *lake* should feel like a set of well-named design cards.
Each card has enough prose to be useful on its own,
but it also participates in a larger graph through metadata.
The graph is intentionally small:
classification, belonging, prerequisites, refinement, and *witnesses*.
That small set is enough to navigate without turning the *lake* into a separate database language.
The `frozen:` marker adds a file-level protection state,
so one current frost-backed lake *entry* can be held read-only
and checked with its artifact tree against the frost snapshot before commit.

The *lake* is also a collaboration boundary.
A person can edit an *entry* directly.
A CLI can check its metadata and links.
An agent can query a few related *entries* before changing code.
An editor can use *generated footers* to expose navigation.
All of those forms use the same filenames and metadata.

The *lake* is a working form.
Direct edits become frozen versions only when frost is configured
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
  - [entry-domain](entry-domain.md)
  - [entry-path](entry-path.md)
  - [lake-sheaf](lake-sheaf.md)
  - [metadata](metadata.md)
  - [query](query.md)
  - [refinement](refinement.md)
  - [sirno-tide](sirno-tide.md)
  - [structural-check](structural-check.md)

> **Sirno generated links end.**
