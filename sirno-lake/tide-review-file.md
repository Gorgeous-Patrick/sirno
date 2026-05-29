---
name: Tide Review File
desc: The target tracked TOML file for active Tide review status.
category:
  - concept
  - proposal
belongs:
  - tide
  - sirno-control-files
prerequisite:
  - tide-resolution
  - anchor-file
  - sirno-control-files
---

The Tide review file is the target `.sirno/tide.toml` design.
It stores active review status against the current Anchor.

The file is target-first.
A reviewer opens one reached entry and reviews it once,
even when several ripples reached that entry.

```toml
schema = 1
anchor = "sha256:..."

[[reviews]]
entry = "methodology"
entry_fingerprint = "sha256:..."
reviewer = "arctic"
reviewed_at = "2026-05-26T14:30:00Z"

[[reviews.reaches]]
ripple = "storage"
ripple_fingerprint = "sha256:..."
field = "belongs"
direction = "from"
```

A review means that the reviewer inspected `entry` at `entry_fingerprint`
and accepted the listed ripples that reached it.
It resolves a current workitem only when these values still match:

- target entry id;
- target entry fingerprint;
- ripple id;
- ripple fingerprint;
- relation field;
- edge direction.

If the target entry changes, its reviews reopen.
If a ripple changes, only reviews for that ripple reopen.
Unrelated edits do not invalidate the review.

The file is active review evidence, not permanent history.
Anchor update consumes it by accepting the current lake into `.sirno/anchor.toml`
and deleting `.sirno/tide.toml`.
The durable accepted record is the new Anchor plus the Git commit.

An explicit archive or export command may later preserve review receipts.
That is future work and should not make active review state permanent by default.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-control-files](sirno-control-files.md)
  - [tide](tide.md)
- belongs (from): (none)

> **Sirno generated links end.**
