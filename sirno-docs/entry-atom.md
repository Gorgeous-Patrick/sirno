---
name: Entry Atom
desc: One dot-free segment in an entry address.
category:
  - concept
belongs:
  - sirno-lake
prerequisite:
  - entry
---

An *entry atom* is one dot-free segment in an *entry address*.

An atom maps to one filesystem component in the lake.
A non-final atom is an *entry domain* segment.
The final atom is the local entry filename stem before `.md`.

Atoms are validated as cross-platform filename stems.
They must not use path separators, control characters, Windows-reserved punctuation,
reserved device names, a trailing space, or `.`.
The `.` character belongs to the address grammar that joins atoms.

Use lowercase ASCII kebab-case by default.
That style is easy to type, quote, link, and compare across tools.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
