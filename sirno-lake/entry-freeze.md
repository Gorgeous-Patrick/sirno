---
name: Entry Freeze
desc: A read-only entry marker that protects current frost-backed Markdown.
category:
  - concept
belongs:
  - entry
  - sirno-frost
prerequisite:
  - entry
  - sirno-frost
---

An *entry freeze* is an explicit protection state for one lake Markdown *entry*
that already matches the current frost snapshot.
The metadata field is canonical `meta.frozen` with a non-empty list of reasons.
`reviewed` is the reason written by manual frost-backed freeze.
`managed` is the reason written by crystallization.
Frost snapshots store these reasons as durable entry metadata.
Committing or checking out a snapshot preserves whether an entry is frozen
and preserves every frozen reason.

`sirno freeze ENTRY_ADDRESS` verifies that the lake *entry* matches current frost,
adds `reviewed`,
and applies local file protection to the *entry* file and artifact tree.
`sirno melt ENTRY_ADDRESS` removes `reviewed`.
It clears local protection only when no other frozen reason remains.
`sirno unfreeze ENTRY_ADDRESS` is an alias for `sirno melt ENTRY_ADDRESS`.
The command pair is the supported way to change the state.

`sirno freeze`, `sirno melt`, `sirno freeze tui`, and `sirno melt tui`
open one terminal UI for entry freeze and melt work.
The UI lists entries, their freeze state, and artifact counts.
It applies freeze or melt to the selected entry only.
`Space` applies the command's default operation.
`f` freezes, `m` melts, `c` refreshes, and `Tab` switches the default operation.
All-project protection repair stays on `sirno freeze --fix-all`.
All-project unsafe clearing stays on `sirno melt --unsafe-all`.

`sirno melt --unsafe-all` clears all Sirno local filesystem protection
from the active lake.
It does not remove `meta.frozen` markers, change lock state, or delete files.
It prints a danger warning and the selected paths.
Use it when protected files must become writable or deletable for external cleanup.
`sirno freeze --fix-all` reapplies local protection from `meta.frozen` reasons
and immutable frost checkout state.
`--dry-run` reports the selected paths without changing permissions.

A frozen *entry* remains visible in the lake for reading, checking, and querying.
The frost layer accepts a frozen *entry* only while its committed form still matches
the current frost snapshot.
Generated-link regions are ignored for this comparison.
The `reviewed` reason is projected out so `sirno freeze` can mark an unchanged entry.
Other frozen reasons remain part of the committed form.
If the frozen *entry* differs,
the frost layer refuses the commit.
Melt the *entry* before intentionally changing it.

File permissions are a local enforcement layer.
Sirno always removes ordinary write permission from protected files and directories.
When the platform and process allow it,
Sirno also applies the stronger immutable file guard.
The frost comparison remains the authoritative protection against committing drift.

Glacier entries are frozen because they carry `managed`.
They may also carry `reviewed` when the upstream entry was reviewed.
Normal melt removes only `reviewed`;
crystallization owns adding and removing `managed`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [entry](entry.md)
  - [sirno-frost](sirno-frost.md)
- belongs (from): (none)

> **Sirno generated links end.**
