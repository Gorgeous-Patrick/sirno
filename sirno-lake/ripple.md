---
name: Ripple
desc: The named delta between two lake states.
category:
  - concept
  - implemented
belongs:
  - sirno-tide
prerequisite:
  - sirno-anchor
---

A *ripple* is the named delta between Anchor and the waterline.
Anchor is the accepted baseline.
The waterline is the current lake.
Sirno reads the waterline from the configured lake path
and strips generated footer regions before comparison.

A *ripple* exists when an *entry* differs between those two states.
The difference may be a changed name, description, prose body, or structural link.
Added and deleted *entries* are ripples too.
Generated footer regions are ignored,
because Anchor stores canonical fingerprints over metadata and prose rather than rendered navigation.

The term fits the *lake* model.
A *lake* is the readable body of project knowledge.
A *ripple* is a visible disturbance in that body:
small enough to inspect locally,
but meaningful because it belongs to a larger surface.

The *ripple* names reviewable difference, not semantic judgment.
Sirno can show what changed and which configured neighbors must be reviewed.
It does not decide whether the new design is correct.

A *ripple fingerprint* is a `sha256:` hash of one *ripple* delta.
Sirno labels the Anchor side and the waterline side,
uses each side's canonical entry fingerprint when present,
treats an absent side as a fixed placeholder,
and hashes the joined text.
The fingerprint identifies what changed, not the moment of change.
It scopes Tide review resolutions to the exact delta a reviewer saw.

A *ripple* produces a wave of *tide workitems* through relation-defined Tide policies.
The wave is the local review surface around a single changed *entry*.
The *tide* is the active worklist created from all current ripples.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-tide](sirno-tide.md)
- belongs (from): (none)

> **Sirno generated links end.**
