---
name: Spell
desc: A ready-to-run script or executable resolved from a charm.
category:
  - concept
  - proposal
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
Hooks and direct operator commands invoke spells.

A spell does not own design intent.
The charm entry owns that intent,
and the charm artifact tree owns the reviewed runnable material.
The spell is derived from the charm so invocation can stay separate from preparation.

Spell identity should include the charm entry address
and the fingerprint of the charm state that produced it.
That identity lets Sirno report which reviewed charm produced a runtime effect.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [extension-system](extension-system.md)
- belongs (from): (none)

> **Sirno generated links end.**
