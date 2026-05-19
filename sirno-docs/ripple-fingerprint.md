---
name: Ripple Fingerprint
desc: The delta hash that scopes a tide resolution to one exact ripple state.
category:
  - concept
belongs:
  - ripple
---

A *ripple fingerprint* is a short hash of one *ripple*'s delta.
Sirno renders the *entry*'s frostline form and waterline form to canonical Markdown,
labels each side,
treats an absent side as a fixed placeholder,
and folds the joined text through a 64-bit FNV-1a hash.
The result is sixteen hex digits.

The fingerprint identifies *what changed*, not the moment of change.
Two edits that produce identical frostline and waterline forms share a fingerprint.
Any change to either side produces a different fingerprint.

The fingerprint is what makes a *tide resolution* precise.
A resolution stores the fingerprint of the *ripple* delta it reviewed.
Sirno derives open *workitems* on demand
and marks one resolved only when a stored resolution matches
the full workitem tuple *and* the current fingerprint of its *ripple*.

This scopes review to the exact delta a reviewer saw.
If the *ripple entry* changes again before commit,
its fingerprint changes,
the old resolution no longer matches,
and the *workitem* reopens.
Changes to the reviewed *neighbor* do not alter the *ripple* fingerprint,
so they never reopen that *workitem*.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [ripple](ripple.md)
- belongs (from): (none)

> **Sirno generated links end.**
