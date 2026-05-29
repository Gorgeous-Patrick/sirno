---
name: Wave
desc: The set of tide workitems produced by one ripple.
category:
  - concept
  - implemented
belongs:
  - sirno-tide
prerequisite:
  - tide
---

A *wave* is the set of *tide workitems* produced by one *ripple*.
It is the local review surface around a single changed *entry*.

Sirno builds a wave by comparing one ripple entry across Anchor and the waterline,
then applying the configured relation entries' tide policies.
Each enabled edge direction can add neighbors from the waterline,
Anchor,
or both.

The wave does not store review state.
It is derived from the current comparison,
the relation order in `Sirno.toml`,
and the relation entries' `meta.ripple.lake` and `meta.ripple.frost` direction lists.
`meta.ripple.frost` is the temporary spelling for Anchor-side review policy.
`Sirno.lock.toml` stores only explicit resolutions for the workitems inside the wave.

When several ripples exist,
each ripple produces its own wave.
The active *tide* is the union of those waves.
If two waves point at the same neighbor,
they remain separate when their `(ripple, field, direction, neighbor)` tuples differ.
Human tide status output can group review entries and full workitem rows by wave with `--by wave`.
The displayed wave is the ripple entry that produced the wave.
Group boundaries use heavy double separators.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-tide](sirno-tide.md)
- belongs (from): (none)

> **Sirno generated links end.**
