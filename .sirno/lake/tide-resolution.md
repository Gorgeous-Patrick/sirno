---
name: Tide Resolution
desc: A persisted explicit review record and its resolve, reopen, and reset lifecycle.
category:
  - concept
  - implemented
belongs:
  - tide
prerequisite:
  - tide
  - ripple
---

A *tide resolution* is a recorded statement that one review obligation was met.
It pairs a *tide workitem* tuple `(ripple, field, direction, neighbor)`
with the *ripple fingerprint* of the delta the reviewer inspected.

Resolutions are the only *tide* state the current implementation persists.
They live in `.sirno/tide.toml` under active Tide state.
Open *workitems* are never stored;
Sirno derives them on demand from the current *waterline* and Anchor.
A *workitem* counts as resolved only when a stored resolution matches
its full tuple and its *ripple*'s current fingerprint.

Recording a resolution means the reviewer accepted the *neighbor*'s current state
given that exact *ripple* delta.
Resolving by *neighbor* id resolves every open *workitem* whose `neighbor` is that *entry*,
which reads as "I reviewed this entry and accept it."
Resolving a full tuple records only that one obligation.

`sirno resolve --infer` resolves every open *workitem* whose `neighbor`
is itself one of the current *ripple* entries.
When the neighbor also changed in the same edit session,
the editor already worked on it directly,
so inference closes the dependency-review obligation without a manual selector.
Inferred resolutions are ordinary *tide resolutions*
and stay bound to the same tuple and *ripple fingerprint*.

`sirno resolve` records review.
`sirno unresolve` removes matching resolutions,
and `sirno reopen` is its alias.
`sirno resolve --infer` applies the mutual-ripple rule.
The grouped forms are `sirno tide resolve`, `sirno tide unresolve`,
and `sirno tide reopen` as an alias of unresolve.
`sirno tide reset` clears every tide resolution from the Tide file at once.

A resolution is bound to its delta, not to wall-clock time.
If the *ripple entry* changes again, its fingerprint changes,
the resolution stops matching, and the obligation reopens.
This keeps acceptance honest without storing a separate worklist.

A clear *tide* gates `sirno anchor update` after Anchor is initialized.
The first `sirno anchor update` initializes Anchor from the current lake.
A later update refuses to accept the lake while any open *workitem* remains.
A successful update clears tide resolutions because the new Anchor makes the prior deltas moot.

The Tide file stores active review records for the current waterline.
Anchor update deletes `.sirno/tide.toml` after accepting the current waterline.
The durable accepted record is the new Anchor plus the Git commit,
not a permanent active-review file.

When `[tutorial]` is configured,
an update blocked by an open *tide* can print the worklist
and explain the empty-Anchor bootstrap case.
