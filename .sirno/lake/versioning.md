---
desc: Git history and Anchor accepted baselines for Sirno Lake changes.
name: Versioning
category:
  - concept
  - implemented
belongs:
  - lake-review
  - anchor
prerequisite:
  - reservoir
  - anchor
---

Sirno delegates repository history to Git.
Git stores commits, branches, rebases, restores, retention, and destructive history operations.
Sirno records the accepted design baseline as `.sirno/anchor.toml`.

The reservoir is the canonical working form.
An Anchor update accepts that working form by writing semantic fingerprints for live entries
and their artifact trees.
The resulting file is tracked by Git with the source changes that made the lake acceptable.

The reservoir and mist design makes the reservoir the versioned lake surface.
Git tracks `.sirno/lake`, Sirno control files, and shared mist specs.
Git does not track misty-lake workspace files.
Sirno guards commits by rejecting staged misty-lake paths
and points the user toward explicit mist intake.

A version of Sirno design is therefore a Git commit that contains:

- `Sirno.toml`;
- `.sirno/lake`;
- shared mist specs when the reservoir and mist design is active;
- `.sirno/anchor.toml`;
- optional active `.sirno/tide.toml` review state;
- optional `.sirno/upstream.toml` dependency pins.

Anchor is not a history store.
It does not contain old entry bodies, restore coordinates, checkout state, or retention policy.
To inspect or restore old design states,
use Git to read or check out the commit that contains those states.

Tide compares the current lake against Anchor.
A ripple is an entry-level delta between the accepted baseline and the current waterline.
Review status is valid only while the involved entry fingerprints and ripple fingerprints match.
After `sirno anchor update`, the current waterline becomes the accepted baseline.
In the target split,
unintaken mist ripples, stale state, conflicts, or staged misty-lake files block Anchor update
until those ripples are intaken or discarded.

Entry artifacts are versioned by Git as ordinary files under the lake `.artifacts` tree.
Anchor stores an owner artifact-tree fingerprint for entries that own artifacts.
This lets Tide notice artifact changes without storing artifact bytes in a private Sirno store.

Sirno-owned control files must remain valid TOML across Git operations.
The *Sirno Control Files* entry defines merge-driver policy for those files.
