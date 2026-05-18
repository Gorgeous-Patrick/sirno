---
name: Tide
desc: The Frost-based worklist of review obligations for lake edits.
category:
  - concept
belongs:
  - frost-versioning
refines:
  - ripple
---

A *tide* is the dependency-based review worklist for the current public *lake*.
It compares the waterline, the current public *lake*,
against the frostline, the latest Sirno Frost snapshot.

Every changed *entry* is a *ripple*.
For each *ripple*, Sirno reads the configured structural edge policies
and produces one wave of review obligations.
The *tide* is the union of those open obligations.

Sirno derives open workitems on demand.
`Sirno.lock.toml` stores only explicit resolutions and their ripple fingerprints.
If the *ripple* entry changes again before commit,
the matching resolution no longer applies.
Changes to the reviewed neighbor do not reopen that workitem.

`sirno tide status` prints one table with a wave column.
Each wave starts with its ripple id and lists the entries that still need review.
Wave boundaries use heavy double separators.
A one-sentence summary follows the table.
`sirno tide status --full` prints full open workitem statuses in the same wave-grouped table.
`sirno tide status --full --all` includes resolved workitem statuses.
`sirno resolve` records explicit review.
`sirno resolve --infer` resolves workitems whose neighbor is also in the current ripple set,
including neighbors deleted from the waterline.
`sirno unresolve` removes matching resolutions.
`sirno reopen` is an alias for `sirno unresolve`.
The grouped forms are `sirno tide resolve` and `sirno tide unresolve`.
`sirno tide reopen` is an alias for `sirno tide unresolve`.
`sirno tide reset` clears all tide resolutions from the lock.

`sirno commit` requires a clear *tide*.
`sirno commit --unsafe-resolve-all` bypasses that gate for the current commit,
does not write fake resolutions,
and clears tide state after a successful commit.
When `[tutorial]` is present in `Sirno.toml`,
an open-*tide* commit failure may explain the worklist
and the empty-version bootstrap case.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning](frost-versioning.md)
- belongs (from): (none)

> **Sirno generated links end.**
