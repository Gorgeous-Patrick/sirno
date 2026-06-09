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

The Sirno Lake is the configured set of Markdown *entries*.
It is the `lake` form in a Sirno-managed project:
canonical, addressable design knowledge that repository material can actualize or witness.
The default reservoir path is `.sirno/lake`.

The name keeps Sirno's Cirno influence visible.
It echoes Misty Lake and recasts the project design source as a Misty Lake of Knowledge:
quiet enough to read,
but structured enough that changes leave visible *ripples*.

The reservoir and mist design keeps *lake* as the conceptual entry set.
The *reservoir* stores the canonical lake.
A *mist* selects entries from the reservoir.
A *misty lake* is the projected workspace that people and agents read,
edit, and receive rendered output in.

The *lake* is a human-readable intermediate representation:
text first, structured enough for tools,
and compact enough for humans and agents to inspect locally.
Anchor records the accepted baseline for the *lake*.
Git versions the *lake* and Anchor file together.
The reservoir is the versioned lake surface.
Misty lakes remain projected working surfaces.

The *lake* should feel like a set of well-named design cards.
Each card has enough prose to be useful on its own,
but it also participates in a larger graph through metadata.
The graph is intentionally small:
classification, belonging, prerequisites, refinement, and *witnesses*.
That small set is enough to navigate without turning the *lake* into a separate database language.

The Sirno Lake entry is the front door for the lake neighborhood.
It routes readers to smaller review neighborhoods
instead of owning every leaf concept directly.
Read these entries when changing the lake model:

| Area | Entry | Local claim |
|---|---|---|
| entry shape | `entry` | Markdown files, metadata, prose, and addressable design objects. |
| canonical store | `reservoir` | The tracked authored lake store. |
| projection | `misty-lake` | Selection, rendered workspaces, generated navigation, and intake. |
| composition | `lake-composition` | Namespaces, lakelets, upstreams, and crystallized glaciers. |
| commands | `lake-commands` | Lake initialization, checking, and reservoir movement. |
| review | `lake-review` | Structural validation, accepted baselines, Tide, and versioning. |

Some files under a *lake* root may belong to adjacent tools.
`[lake].ignore` lists paths relative to the reservoir root.
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
