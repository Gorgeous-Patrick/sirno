---
name: Infer Resolution
desc: Resolving tide workitems whose neighbor also changed in the current ripple set.
category:
  - concept
belongs:
  - tide
prerequisite:
  - tide-resolution
---

*Infer resolution* is `sirno resolve --infer`.
It resolves every open *tide workitem* whose `neighbor`
is itself one of the current *ripple* entries.

The rule encodes a common review situation.
When a *neighbor* also changed in the same edit session,
the editor already worked on it directly.
The obligation to revisit it as a dependency is then redundant,
so inference closes it without a manual selector.

Inference reads the current *ripple* set,
which includes *entries* deleted from the *waterline*.
A *workitem* pointing at a deleted *neighbor* is therefore inferable,
so removing an *entry* and clearing its inbound review can happen in one step.

Inferred resolutions are ordinary *tide resolutions*.
They store the same tuple and *ripple fingerprint*,
so a later change to the *ripple entry* still reopens them.
Inference chooses *which* obligations to resolve;
it does not weaken how a resolution is bound to its delta.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [tide](tide.md)
- belongs (from): (none)

> **Sirno generated links end.**
