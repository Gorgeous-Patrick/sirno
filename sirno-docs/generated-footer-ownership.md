---
name: Generated Footer Ownership
desc: The guard-bounded boundary between Sirno-owned navigation and user-owned prose.
category:
  - concept
belongs:
  - generated-navigation
prerequisite:
  - generated-footer
---

Generated footer ownership is the rule that Sirno mutates only the guard-bounded region.

The opening and closing sentinels are part of the owned region.
Commands that create, replace, check, or delete rendered generated links validate that region first.
Malformed, missing, duplicated, or reversed sentinels are structural errors.

Prose outside the generated-link region remains user-owned.
Mutating render commands preserve that prose.
A *frost* commit removes generated-link regions before writing snapshots,
so Sirno Frost keeps canonical metadata and prose rather than navigation projections.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation](generated-navigation.md)
- belongs (from): (none)

> **Sirno generated links end.**
