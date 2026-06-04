---
name: Control Files
desc: The shared contract for Sirno-owned TOML files under .sirno.
category:
  - concept
  - proposal
belongs:
  - project-config
prerequisite:
  - project-config
refines:
  - project-config
---

Control files are Sirno-owned TOML files under `.sirno/` next to `Sirno.toml`.
They store generated or semi-generated project state that must be shared through Git.

`Sirno.toml` stays at the repository root because it marks the project
and carries human-authored configuration.
`.sirno/` groups Sirno-owned state without hiding the project marker.
The `.sirno/` path is fixed.
A fixed path keeps status output, documentation, and merge-driver setup stable.

This entry owns the shared directory rule,
the Git-tracked TOML rule,
and the merge-validity rule.
The file-specific entries own schema, lifecycle, write path, and subsystem meaning.

| File | Owner | Role |
|---|---|---|
| `.sirno/anchor.toml` | `anchor-file` | Accepted lake baseline. |
| `.sirno/meta.toml` | `meta-registry` | Generated meta registry lockfile. |
| `.sirno/tide.toml` | `tide-review-file` | Active Tide review state. |
| `.sirno/upstream.toml` | `upstream-file` | Generated upstream dependency pins. |

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
When semantic merge is unsafe,
the driver must keep a deterministic complete file
and leave explicit follow-up work to Sirno or the user.

This rule keeps destructive Git operations conservative.
The file remains parseable,
and Sirno commands report semantic drift or open Tide work instead of failing on broken TOML.
