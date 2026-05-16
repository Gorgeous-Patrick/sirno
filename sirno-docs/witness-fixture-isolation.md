---
name: Witness Fixture Isolation
desc: Test fixtures for witness lookup avoid depending on their own delimiter text.
category:
  - concept
belongs:
  - witness
refines:
  - witness-lookup
---

Witness fixture isolation keeps tests for *witness* lookup from satisfying themselves through source literals.

Tests that need an actual *repository witness* fixture assemble the delimiter text from smaller parts
before writing temporary files.
The scanner still sees a real *witness* block in the fixture file.
The Rust test source does not expose that fixture block as a standalone string literal.

Tests that only format *witness* records use neutral comment text.
They verify range rendering, marker selection, body preservation,
and record spacing without depending on *witness* syntax.

Repository *witness* comments remain valid evidence in test modules.
They are *repository* metadata, not fixture data.
The isolation rule applies to strings and generated files that a test creates for the scanner
or formatter.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [witness](witness.md)
- belongs (from): (none)

> **Sirno generated links end.**
