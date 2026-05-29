---
name: Sirno Control Files
desc: The tracked Sirno-owned TOML files stored under .sirno.
category:
  - concept
  - proposal
belongs:
  - storage
prerequisite:
  - storage
  - project-config
  - anchor-file
  - sirno-lock
  - tide-resolution
refines:
  - storage
---

Sirno control files are tracked TOML files under `.sirno/` next to `Sirno.toml`.
They store generated or semi-generated project state.

`Sirno.toml` stays at the repository root because it marks the project
and carries human-authored configuration.
`.sirno/` groups Sirno-owned state without hiding the project marker.
The path is fixed.
A fixed path keeps status output, documentation, and merge-driver setup simple.

The target control directory is:

```text
.sirno/
  anchor.toml
  tide.toml
  lock.toml
```

`.sirno/anchor.toml` records the accepted lake baseline.
It exists after the first successful Anchor update.

`.sirno/tide.toml` records active Tide review status for the current diff.
It exists only while review status must survive across commands or Git operations.
Anchor update deletes it after accepting the waterline.

`.sirno/lock.toml` records external upstream dependency pins.
It exists only when the project has shared upstream pins.

The current implementation has only `.sirno/anchor.toml` in this directory.
`Sirno.lock.toml` still stores upstream pins and Tide resolutions
until `.sirno/tide.toml` and `.sirno/lock.toml` are actualized.

## Merge Validity

Sirno control files are tracked by Git,
so merges, rebases, cherry-picks, and conflict resolution may touch them.
Sirno should install merge drivers for the target files:

```gitattributes
.sirno/anchor.toml merge=sirno-anchor
.sirno/tide.toml merge=sirno-tide
.sirno/lock.toml merge=sirno-lock
```

A Sirno merge driver must write complete valid TOML.
It must not leave conflict markers in a control file.
When the driver cannot prove that a review still matches current fingerprints,
it drops that review and lets Tide reopen the obligation.
When the driver cannot safely merge dependency pins,
it keeps a deterministic complete state and leaves explicit update work to the user.

This rule keeps destructive Git operations conservative.
The file remains parseable,
and Sirno commands report semantic drift or open Tide work instead of failing on broken TOML.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [storage](storage.md)
- belongs (from):
  - [tide-review-file](tide-review-file.md)

> **Sirno generated links end.**
