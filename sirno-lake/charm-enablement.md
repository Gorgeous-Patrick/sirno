---
name: Charm Enablement
desc: The local opt-in policy that makes a charm executable project policy.
category:
  - concept
  - proposal
belongs:
  - extension-system
  - project-config
prerequisite:
  - charm
  - spell
  - project-config
  - entry-freeze
  - sirno-anchor
refines:
  - extension-system
---

*Charm enablement* is the local opt-in decision that allows Sirno to resolve and invoke a charm.

A project enables charm entries in `Sirno.toml`,
not in the artifact tree alone.
A present charm manifest is discoverable design data.
An enabled charm is executable project policy.

The operator grants each spell resolved from an enabled charm
the same filesystem authority as the Sirno process.
Sirno should make that authority explicit in CLI output and config comments.
It should not imply sandboxing unless a later implementation provides one.

Frozen entries protect the charm artifact tree from accidental mutation.
They do not make the code trusted.
Trust comes from local enablement and ordinary repository review.
Anchor and Tide already track owner artifact-tree fingerprints,
so charm code changes participate in lake review like other entry artifact changes.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [extension-system](extension-system.md)
  - [project-config](project-config.md)
- belongs (from): (none)

> **Sirno generated links end.**
