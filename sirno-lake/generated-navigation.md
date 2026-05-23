---
name: Generated Navigation
desc: The review neighborhood for Sirno-owned generated footer navigation.
category:
  - concept
belongs:
  - structural
prerequisite:
  - generated-footer
---

Generated navigation is the review front door for the Sirno-owned footer machinery.

It gathers the parts that produce and bound rendered generated links:
`generated-footer` is the footer Sirno projects from metadata,
`generated-footer-ownership` is the guard-bounded boundary Sirno may mutate,
and `structural-edge-policy` chooses which structural links appear.

These parts are reviewed together.
A change to footer rendering, ownership boundaries, or structural link policy
usually constrains the others, so this *entry* gives them one neighborhood.

`generated-footer` remains the broader claim ownership and policy `refines`.
This neighborhood is the separate horizontal view:
`refines` says what a part specializes,
`belongs` here says which parts are reviewed together.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [structural](structural.md)
- belongs (from):
  - [generated-footer](generated-footer.md)
  - [generated-footer-ownership](generated-footer-ownership.md)
  - [structural-edge-policy](structural-edge-policy.md)

> **Sirno generated links end.**
