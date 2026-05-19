---
name: Tide Resolution
desc: A persisted explicit review record and its resolve, reopen, and reset lifecycle.
category:
  - concept
belongs:
  - tide
---

A *tide resolution* is a recorded statement that one review obligation was met.
It pairs a *tide workitem* tuple `(ripple, field, direction, neighbor)`
with the *ripple fingerprint* of the delta the reviewer inspected.

Resolutions are the only *tide* state Sirno persists.
They live in `Sirno.lock.toml` under tide state.
Open *workitems* are never stored;
Sirno derives them on demand from the current *waterline* and *frostline*.
A *workitem* counts as resolved only when a stored resolution matches
its full tuple and its *ripple*'s current fingerprint.

Recording a resolution means the reviewer accepted the *neighbor*'s current state
given that exact *ripple* delta.
Resolving by *neighbor* id resolves every open *workitem* whose `neighbor` is that *entry*,
which reads as "I reviewed this entry and accept it."
Resolving a full tuple records only that one obligation.

`sirno resolve` records review.
`sirno unresolve` removes matching resolutions,
and `sirno reopen` is its alias.
`sirno resolve --infer` applies the mutual-ripple rule;
see *infer resolution*.
The grouped forms are `sirno tide resolve`, `sirno tide unresolve`,
and `sirno tide reopen` as an alias of unresolve.
`sirno tide reset` clears every tide resolution from the lock at once.

A resolution is bound to its delta, not to wall-clock time.
If the *ripple entry* changes again, its fingerprint changes,
the resolution stops matching, and the obligation reopens.
This keeps acceptance honest without storing a separate worklist.

A clear *tide* is a *frost* commit gate.
`sirno commit` refuses to freeze the *lake* while any open *workitem* remains.
`sirno commit --unsafe-resolve-all` bypasses the gate for that one commit.
It writes no fake resolutions and clears tide state after the commit succeeds.
A normal successful commit also clears tide resolutions,
because the new *frostline* makes the prior deltas moot.

When `[tutorial]` is configured,
a commit blocked by an open *tide* can print the worklist
and explain the empty-*frostline* bootstrap case.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [tide](tide.md)
- belongs (from): (none)

> **Sirno generated links end.**
