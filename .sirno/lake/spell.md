---
name: Spell
desc: A ready-to-run script or executable resolved from a charm.
category:
  - concept
  - implemented
belongs:
  - extension-system
prerequisite:
  - charm
refines:
  - extension-system
---

A *spell* is the ready-to-run script or executable resolved from a *charm*.

A spell may be a direct artifact in the charm tree.
It may also be a build output written to Sirno cache state.
In either case,
the spell is the runtime object.
Direct operator commands invoke spells today.
Hooks should invoke spells after the hook design is accepted.

A spell does not own design intent.
The charm entry owns that intent,
and the charm artifact tree owns the reviewed runnable material.
The spell is derived from the charm so invocation can stay separate from preparation.

Spell identity should include the charm entry address
and the fingerprint of the charm state that produced it.
That identity lets Sirno report which reviewed charm produced a runtime effect.
