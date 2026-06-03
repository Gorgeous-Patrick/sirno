---
name: Sirno Control Files
desc: The Sirno-owned TOML files stored under .sirno.
category:
  - concept
  - proposal
belongs:
  - storage
prerequisite:
  - storage
  - project-config
  - anchor-file
  - upstream-file
  - tide-resolution
refines:
  - storage
---

Sirno control files are TOML files under `.sirno/` next to `Sirno.toml`.
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
  meta.toml
  tide.toml
  upstream.toml
```

`.sirno/anchor.toml` records the accepted lake baseline.
It exists after the first successful Anchor update.

`.sirno/meta.toml` records the generated meta registry for the current lake.
It is a tracked lockfile.
Sirno rewrites it from raw entry metadata when the registry changes.

`.sirno/tide.toml` records active Tide review status for the current diff.
It exists only while review status must survive across commands or Git operations.
Anchor update deletes it after accepting the waterline.

`.sirno/upstream.toml` records external upstream dependency pins.
It exists only when the project has shared upstream pins.

The current implementation stores Anchor, meta registry, Tide, and upstream control state
in this directory.

## Merge Validity

Tracked Sirno control files are tracked by Git,
so merges, rebases, cherry-picks, and conflict resolution may touch them.
Sirno should install merge drivers for the target files:

```gitattributes
.sirno/anchor.toml merge=sirno-anchor
.sirno/meta.toml merge=sirno-meta
.sirno/tide.toml merge=sirno-tide
.sirno/upstream.toml merge=sirno-upstream
```

A Sirno merge driver must write complete valid TOML.
It must not leave conflict markers in a control file.
When the meta registry cannot be merged directly,
it regenerates the lockfile from the merged lake entries.
When the driver cannot prove that a review still matches current fingerprints,
it drops that review and lets Tide reopen the obligation.
When the driver cannot safely merge dependency pins,
it keeps a deterministic complete state and leaves explicit update work to the user.

This rule keeps destructive Git operations conservative.
The file remains parseable,
and Sirno commands report semantic drift or open Tide work instead of failing on broken TOML.
