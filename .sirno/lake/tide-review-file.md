---
name: Tide Review File
desc: The tracked TOML file for active Tide resolution state.
category:
  - concept
  - implemented
belongs:
  - tide
  - sirno-control-files
prerequisite:
  - tide-resolution
  - anchor-file
  - sirno-control-files
---

The Tide review file is `.sirno/tide.toml`.
It stores active review status against the current Anchor.
It exists when explicit Tide resolutions must survive across commands or Git operations.

The file stores the current resolution set with schema version `1`.
Each `resolved` item records the workitem tuple and the ripple fingerprint it reviewed.

```toml
schema = 1

[[resolved]]
ripple = "storage"
field = "belongs"
direction = "from"
neighbor = "methodology"
fingerprint = "sha256:..."
```

A resolution means the reviewer accepted `neighbor`
for the listed relation edge from `ripple`.
It resolves a current workitem only when these values still match:

- ripple id;
- ripple fingerprint;
- relation field;
- edge direction;
- neighbor entry id.

If a ripple changes, only resolutions for that ripple reopen.
Unrelated edits do not invalidate the resolution.

The file is active review evidence, not permanent history.
Anchor update consumes it by accepting the current lake into `.sirno/anchor.toml`
and deleting `.sirno/tide.toml`.
The durable accepted record is the new Anchor plus the Git commit.

An explicit archive or export command may later preserve review receipts.
That is future work and should not make active review state permanent by default.
