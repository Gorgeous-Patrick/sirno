---
name: Ripple
desc: The named delta between two lake states.
category:
  - concept
  - implemented
belongs:
  - sirno-tide
prerequisite:
  - anchor
  - waterline
---

A *ripple* is the named delta between Anchor and the waterline.
Anchor is the accepted baseline.
The waterline is the current lake.

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

A *ripple* produces a *wave* of *tide workitems* through relation-defined tide policies.
The *tide* is the active worklist created from all current ripples.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-tide](sirno-tide.md)
- belongs (from):
  - [ripple-fingerprint](ripple-fingerprint.md)
  - [waterline](waterline.md)

> **Sirno generated links end.**
