---
name: Generated Link Policy
desc: Configuration that chooses which structural links appear in generated footers.
category:
  - concept
belongs:
  - generated-navigation
refines:
  - generated-footer
---

Generated link policy decides which configured structural sections appear in a generated footer.

`[structural]` lists the metadata fields Sirno treats as structural.
Each field key has a `link` policy.
`link.to` generates outgoing links to targets.
`link.from` generates incoming links from sources.
`link.clique` generates clique links through shared targets in that field.
All three booleans are optional,
and an absent boolean means false.

Clique projection does not change direct `from` or `to` projection.
When enabled for a field, each target induces clique edges:
the target links to its members,
and members link to the target and to one another.
When disabled, only configured direct structural field sections are rendered.

This policy is configuration, not *entry* data.
Changing it alters generated navigation surfaces without changing structural metadata.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation](generated-navigation.md)
- belongs (from): (none)

> **Sirno generated links end.**
