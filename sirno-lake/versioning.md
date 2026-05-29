---
name: Versioning
desc: Git history and Anchor accepted baselines for Sirno Lake changes.
category:
  - concept
  - implemented
belongs:
  - sirno-lake
  - anchor
prerequisite:
  - storage
  - anchor
---

Sirno delegates repository history to Git.
Git stores commits, branches, rebases, restores, retention, and destructive history operations.
Sirno records the accepted design baseline as `.sirno/anchor.toml`.

The editable lake is the working form.
An Anchor update accepts that working form by writing semantic fingerprints for live entries
and their artifact trees.
The resulting file is tracked by Git with the source changes that made the lake acceptable.

A version of Sirno design is therefore a Git commit that contains:

- `Sirno.toml`;
- the Sirno Lake;
- `.sirno/anchor.toml`;
- optional `.sirno/lock.toml` or `Sirno.lock.toml` dependency pins.

Anchor is not a history store.
It does not contain old entry bodies, restore coordinates, checkout state, or retention policy.
To inspect or restore old design states,
use Git to read or check out the commit that contains those states.

Tide compares the current lake against Anchor.
A ripple is an entry-level delta between the accepted baseline and the current waterline.
Review status is valid only while the involved entry fingerprints and ripple fingerprints match.
After `sirno anchor update`, the current waterline becomes the accepted baseline.

Entry artifacts are versioned by Git as ordinary files under the lake `.artifacts` tree.
Anchor stores an owner artifact-tree fingerprint for entries that own artifacts.
This lets Tide notice artifact changes without storing artifact bytes in a private Sirno store.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [anchor](anchor.md)
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
