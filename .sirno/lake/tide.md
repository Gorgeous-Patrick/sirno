---
name: Tide
desc: The Anchor-based worklist of review obligations for lake edits.
category:
  - concept
  - implemented
belongs:
  - sirno-tide
prerequisite:
  - sirno-tide
---

A *tide* is the dependency-based review worklist for the current lake.
It compares the *waterline*, the current lake,
against Anchor, the accepted baseline.

Every changed *entry* is a *ripple*.
For each *ripple*, Sirno reads the relation entries' tide policies
and produces one *wave* of *tide workitems*.
The *tide* is the union of those open obligations.

```mermaid
flowchart LR
    anchor([anchor])
    waterline([waterline])
    ripple([ripple])
    wave([wave])
    tide([tide])
    anchor --> ripple
    waterline --> ripple
    ripple -->|relation tide policy| wave
    wave --> tide
```

Sirno derives open *workitems* on demand.
It stores no worklist;
the current implementation keeps only *tide resolutions* in `.sirno/tide.toml`,
each scoped to a *ripple fingerprint*.
That binding is what reopens an obligation when its *ripple* changes again.

The Tide review file is deleted after Anchor accepts the waterline.

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
the `anchor update` gate,
and the inference shortcut are *tide resolution* behavior.
