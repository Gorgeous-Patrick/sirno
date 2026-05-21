---
name: Witness Lookup
desc: The mosaika-backed scan that resolves witness blocks by entry id.
category:
  - concept
belongs:
  - sirno-witness
prerequisite:
  - witness-delimiter
  - project-config
---

The *witness* lookup resolves *repository* evidence by scanning configured repo members with `mosaika`.
The CLI resolves the requested *entry* id in the active *lake* before scanning.
Missing *entries* fail before repo members are read.

`[repo].members` defines the *repository* artifact surface when *witness* lookup is configured.
File members are scanned directly.
Directory members are scanned recursively.
Glob members expand to matching files.

Sirno projects each member file into a `mosaika` *transform* that analyzes *witness* blocks.
The opening and closing delimiters both capture the *entry* id.
Sirno rejects a *witness* block when the delimiter ids differ.
The delimiter regex pairs come from `[[witness.delimiters]]` config tables.
An empty delimiter list disables witness scanning and yields no witness records.
Generated configs write the standard syntax,
which accepts `//` line comments and hidden Markdown HTML comments.
Those standard regexes share one canonical capture for filename-like *entry* ids.
Sirno reads `mosaika` match records into *witness* records keyed by *entry* id.
The stored delimiter spans exclude leading indentation.
Full output displays every line spanned by the matched block
and preserves the matched text.
The checker also scans the configured delimiter regexes as individual tokens.
A delimiter token that is not consumed by a resolved block is reported as an orphan delimiter.

When `sirno entry rename` updates *witness* sentinels,
Sirno builds `mosaika` text edits from the captured id spans in the opening and closing delimiters.
Only the captured ids are rewritten.
The *witness* body remains owned by the repository artifact.
This makes an *entry* id a safe thing to change:
the id is the single link between a claim and its evidence,
so renaming the claim updates exactly that link without touching the witnessed code.

Tests for *witness* lookup must avoid satisfying themselves through their own source literals.
A fixture file that contains a literal opening or closing sentinel would be picked up by the scanner
as a real *witness* block.
Tests that need a real fixture assemble the delimiter text from smaller parts at runtime
before writing temporary files.
Tests that only format *witness* records use neutral comment text
to verify range rendering, marker selection, body preservation, and record spacing
without depending on *witness* syntax.
The convention applies to fixture data the test writes for the scanner or formatter,
not to *repository witness* comments inside test modules,
which remain valid evidence.

The lookup path keeps *witness* syntax out of *entry* prose.
The *entries* remain design claims.
Repository artifacts carry the precise source spans that witness those claims.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-witness](sirno-witness.md)
- belongs (from): (none)

> **Sirno generated links end.**
