---
name: Entry Freeze
description: A read-only entry marker that keeps protected Markdown out of Frost commits.
category:
  - concept
belongs:
  - sirno-lake
---

An *entry freeze* is an explicit protection state for one public Markdown *entry*.
The metadata marker is canonical `frozen:` with no value.

`sirno freeze ENTRY_ID` adds the marker and removes write permission from the entry file.
`sirno melt ENTRY_ID` removes the marker and restores write permission.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.
The command pair is the supported way to change the state.

A frozen *entry* remains visible in the public *lake* for reading, checking, and querying.
Sirno Frost refuses to commit an *entry* carrying `frozen:`.
Melt the *entry* before creating a Frost snapshot from it.

File permissions are a local enforcement layer.
On Unix, freezing removes the write bits from the entry file.
On other platforms, Sirno uses the platform read-only flag when the filesystem supports it.

---

> **Sirno generated links begin. Do not edit this section.**

belongs (to):
- [sirno-lake](sirno-lake.md)

belongs (from): (none)

> **Sirno generated links end.**
