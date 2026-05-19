---
name: Entry Freeze
desc: A read-only entry marker that protects current Frost-backed Markdown.
category:
  - concept
belongs:
  - entry
  - sirno-frost
---

An *entry freeze* is an explicit protection state for one public Markdown *entry*
that already matches the current Sirno Frost snapshot.
The metadata marker is canonical `frozen:` with no value.

`sirno freeze ENTRY_ID` verifies that the public *entry* matches current Frost,
adds the marker,
and applies local file protection to the *entry* file and artifact tree.
`sirno melt ENTRY_ID` removes the marker and clears that local protection.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.
The command pair is the supported way to change the state.

A frozen *entry* remains visible in the public *lake* for reading, checking, and querying.
Sirno Frost accepts a frozen *entry* only while its committed form still matches
the current Frost snapshot.
Generated-link regions and the `frozen:` marker are ignored for this comparison.
If the frozen *entry* differs,
Frost refuses the commit.
Melt the *entry* before intentionally changing it.

File permissions are a local enforcement layer.
Sirno always removes ordinary write permission from protected files and directories.
When the platform and process allow it,
Sirno also applies the stronger immutable file guard.
Frost comparison remains the authoritative protection against committing drift.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [entry](entry.md)
  - [sirno-frost](sirno-frost.md)
- belongs (from): (none)

> **Sirno generated links end.**
