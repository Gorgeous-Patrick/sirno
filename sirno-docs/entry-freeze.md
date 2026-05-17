---
name: Entry Freeze
desc: A read-only entry marker that protects current Frost-backed Markdown.
category:
  - concept
belongs:
  - frost-versioning
---

An *entry freeze* is an explicit protection state for one public Markdown *entry*
that already matches the current Sirno Frost snapshot.
The metadata marker is canonical `frozen:` with no value.

`sirno freeze ENTRY_ID` verifies that the public *entry* matches current Frost,
adds the marker,
and removes write permission from the *entry* file.
`sirno melt ENTRY_ID` removes the marker and restores write permission.
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
On Unix, freezing removes the write bits from the *entry* file.
On other platforms, Sirno uses the platform read-only flag when the filesystem supports it.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning](frost-versioning.md)
- belongs (from): (none)

> **Sirno generated links end.**
