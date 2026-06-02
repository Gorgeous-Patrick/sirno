---
name: Charm Resolution
desc: The process that prepares a charm and produces a spell.
category:
  - concept
  - implemented
belongs:
  - extension-system
prerequisite:
  - charm
  - spell
  - charm-manifest
refines:
  - extension-system
---

*Charm resolution* turns a charm into a spell.

A direct charm resolves an artifact script or executable without a build.
Sirno resolves the spell command against the spell cache, the owner artifact root,
and the project root.
For charm setup, check, and build commands, the owner artifact root is preferred first.
For spell invocation, the spell cache is preferred first.

A source charm declares `charm.build.command`.
Sirno builds it before the first invocation that needs the spell.
The build writes spell output under `.sirno/spells/<entry-address>/<fingerprint>/`.
The fingerprint includes the entry metadata and artifact tree.

Build output is cache state, not lake state.
Changing charm source, manifest fields, or build-relevant metadata invalidates the cache.
A failed build fails the hook invocation that required it.
A failed build also fails direct `sirno spell run`.

Before a build,
Sirno may run setup or check commands declared by the charm manifest.
Those commands prepare or validate the charm.
They do not invoke the spell unless the hook or operator requested invocation.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [extension-system](extension-system.md)
- belongs (from): (none)

> **Sirno generated links end.**
