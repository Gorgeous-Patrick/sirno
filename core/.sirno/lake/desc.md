---
name: Description
desc: The required plain-string summary field for entries.
meta.type: "intrinsic"
category:
  - meta
  - concept
belongs:
  - category
---

`desc` gives an entry its compact summary.

The value is a single-line plain string.
It should say what the entry is,
not narrate why it was edited.
Sirno uses it in query output,
status surfaces,
and human review routes where opening the full entry would be too much.

This entry exists so the core lake can define the standard summary field for itself.
A project that imports the core lake can still keep its own intrinsic field registry.
