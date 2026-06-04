---
desc: The mosaika-backed scan that resolves witness blocks by entry address.
name: Witness Lookup
category:
  - concept
belongs:
  - witness
prerequisite:
  - witness-delimiter
  - project-config
---

The *witness* lookup resolves *repository* evidence by scanning configured repo members with `mosaika`.
The CLI resolves the requested *entry address* in the active *lake* before scanning.
Missing *entries* fail before repo members are read.

`[repo].members` defines the *repository* artifact surface when *witness* lookup is configured.
File members are scanned directly.
Directory members are scanned recursively.
Glob members expand to matching files.

Sirno projects each member file into a balanced `mosaika` *transform*
that analyzes *witness* blocks.
The opening and closing delimiters both capture the *entry address*.
Sirno rejects a *witness* block when the delimiter paths differ.
The delimiter regex pairs come from `[[witness.delimiters]]` config tables.
An empty delimiter list disables witness scanning and yields no witness records.
Generated configs write the standard syntax,
which accepts `//` line comments and hidden Markdown HTML comments.
Those standard regexes share one canonical capture for valid *entry addresses*.
Sirno reads `mosaika` match records into *witness* records keyed by *entry address*.
One record carries the entry address, repository file path,
full block region, opening and closing delimiter spans,
matched opening delimiter text, and full block body.
The address is parsed from the opening delimiter,
so a record always names a valid *entry*.
The body is emitted by `mosaika` and stays owned by the repository artifact;
Sirno reads it but never authors it.

Record spans are one-based line and column ranges.
The region span covers the whole block.
Delimiter spans cover only the sentinels and exclude leading indentation,
so an indented sentinel still resolves to the comment marker column.
One *entry* may have several records across files
because evidence for one claim can live in more than one place.
`sirno witness ENTRY_ADDRESS --full` prints every record for that address,
showing every line the block spans and preserving the matched text.

Nested *witness* blocks resolve as separate records,
because the balanced matcher pairs each closing delimiter with the nearest open delimiter.
The checker also scans the configured delimiter regexes as individual tokens.
A delimiter token that is not consumed by a resolved block is reported as an orphan delimiter.

When `sirno entry rename` updates *witness* sentinels,
Sirno builds `mosaika` text edits from the captured path spans in the opening and closing delimiters.
Only the captured paths are rewritten.
The *witness* body remains owned by the repository artifact.
This makes an *entry address* a safe thing to change:
the path is the single link between a claim and its evidence,
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
