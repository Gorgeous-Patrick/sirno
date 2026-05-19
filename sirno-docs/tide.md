---
name: Tide
desc: The Frost-based worklist of review obligations for lake edits.
category:
  - concept
belongs:
  - sirno-tide
refines:
  - ripple
---

A *tide* is the dependency-based review worklist for the current public *lake*.
It compares the *waterline*, the current public *lake*,
against the *frostline*, the latest Sirno Frost snapshot.

Every changed *entry* is a *ripple*.
For each *ripple*, Sirno reads the configured structural edge policies
and produces one *wave* of *tide workitems*.
The *tide* is the union of those open obligations.

Sirno derives open *workitems* on demand.
It stores no worklist;
`Sirno.lock.toml` keeps only *tide resolutions*,
each scoped to a *ripple fingerprint*.
That binding is what reopens an obligation when its *ripple* changes again.

`sirno tide status` reports the open worklist.
It prints one table grouped by the *entry* that needs review.
The reason column lists the *ripple* entries whose changes created the obligations.
Group boundaries use heavy double separators,
and a one-sentence summary follows the table.
`sirno tide status --by wave` groups output by *wave* instead.
`sirno tide status --show full` prints full open *workitem* statuses
in the same grouped table,
and `--show all` includes resolved statuses.

Resolving, reopening, and resetting the worklist,
the *frost* commit gate,
and the inference shortcut are *tide resolution* behavior.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-tide](sirno-tide.md)
- belongs (from):
  - [infer-resolution](infer-resolution.md)
  - [tide-resolution](tide-resolution.md)
  - [tide-workitem](tide-workitem.md)

> **Sirno generated links end.**
