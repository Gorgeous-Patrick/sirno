---
name: Entry Freeze
desc: A local entry marker that protects selected Markdown and artifact files.
category:
  - concept
  - deprecated
belongs:
  - entry
prerequisite:
  - entry
---

An *entry freeze* is an explicit protection state for one lake Markdown *entry*.
The metadata field is canonical `meta.frozen` with a non-empty list of reasons.
`managed` is the reason written by crystallization.
`reviewed` is an older manual protection reason and remains deprecated.

`sirno melt ENTRY_ADDRESS` removes `reviewed`.
It clears local protection only when no other frozen reason remains.
`sirno unfreeze ENTRY_ADDRESS` is an alias for `sirno melt ENTRY_ADDRESS`.

`sirno freeze`, `sirno melt`, `sirno freeze tui`, and `sirno melt tui`
open one terminal UI for entry freeze and melt work.
The UI lists entries, their freeze state, artifact counts,
and a summary of discovered intrinsic metadata.
It applies freeze or melt to the selected entry only.
`Space` applies the command's default operation.
`f` freezes, `m` melts, `c` refreshes, and `Tab` switches the default operation.
All-project protection repair stays on `sirno freeze --fix-all`.
All-project unsafe clearing stays on `sirno melt --unsafe-all`.

`sirno melt --unsafe-all` clears all Sirno local filesystem protection
from the active lake.
It does not remove `meta.frozen` markers, change review state, or delete files.
It prints a danger warning and the selected paths.
Use it when protected files must become writable or deletable for external cleanup.
`sirno freeze --fix-all` reapplies local protection from `meta.frozen` reasons.
`--dry-run` reports the selected paths without changing permissions.

File permissions are a local enforcement layer.
Sirno always removes ordinary write permission from protected files and directories.
When the platform and process allow it,
Sirno also applies the stronger immutable file guard.

Glacier entries are frozen because they carry `managed`.
They may also carry `reviewed` when the upstream entry was reviewed.
Normal melt removes only `reviewed`;
crystallization owns adding and removing `managed`.
