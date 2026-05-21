---
name: Witness Record
desc: The resolved repository evidence object for one witness block.
category:
  - concept
belongs:
  - sirno-witness
prerequisite:
  - witness-lookup
---

A *witness record* is one resolved *repository witness* block.
It is what `mosaika` produces after it finds a delimited region
and Sirno parses the captured *entry* id.

A record carries the *entry* id, the repository file path,
the full block region, the opening and closing delimiter *spans*,
the matched opening delimiter text, and the full block body.
The id is parsed from the opening delimiter,
so a record always names a valid *entry*.
The body is emitted by `mosaika` and stays owned by the repository artifact;
Sirno reads it but never authors it.

Spans are one-based line and column ranges.
The region span covers the whole block.
The delimiter spans cover only the sentinels and exclude leading indentation,
so a sentinel written under indented code still resolves
and its start column points at the comment marker rather than the line start.
A span is positional, not semantic;
a reviewer reading the spanned lines still judges whether the evidence is correct.

Records are keyed by *entry* id in a *witness* index.
One *entry* may have several records across files,
because evidence for a claim can live in more than one place.
`sirno witness ENTRY_ID --full` prints the records for that id,
showing every line the block spans and preserving the matched text.

The record is the unit that connects a design claim to its evidence.
The *entry* states the claim in project language.
The record points at the exact region a reviewer should read.
The id is the only link between them.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-witness](sirno-witness.md)
- belongs (from): (none)

> **Sirno generated links end.**
