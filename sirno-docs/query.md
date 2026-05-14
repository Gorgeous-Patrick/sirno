---
name: Query
description: Selection of Sirno entries through vague text and exact structural predicates.
category:
  - concept
belongs:
  - sirno-lake
refines:
  - storage-and-interfaces
---

Query selects parsed entries from the public lake or,
when Sirno Frost is configured,
from one Frost version.

It reads entry ids, metadata, and bodies.
Generated footers are projections for navigation,
not structural input to query.
When no version is supplied,
query reads the public lake.

The default query mode is vague text query.
It matches an entry's id, name, description, and body.
It also matches the ids, names, and descriptions of entries named by the entry's structural fields.

Vague query is for recall.
A user can search for nearby language without choosing the exact structural field first.
Each text term must match somewhere in the expanded entry text.

Exact query uses explicit exact flags.
Exact structural fields are conjunctive across fields and disjunctive inside one field.
Two category values mean either category.
A category plus a refines requires both fields to match.

Query output is presentation.
The same selected entries may be printed as summaries, ids, or paths.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to):
- [sirno-lake](sirno-lake.md)

> **Sirno generated links end.**
