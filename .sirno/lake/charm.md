---
name: Charm
desc: An entry-owned artifact bundle that can resolve into a spell.
category:
  - concept
  - proposal
belongs:
  - extension-system
  - entry-artifact
prerequisite:
  - extension-system
  - entry-artifact
refines:
  - extension-system
---

A *charm* is an *entry* whose artifact tree declares one runnable bundle.

The owning *entry* states the design intent for the charm.
The owner artifact tree stores the material that actualizes that intent.
That material may be a script, a native executable,
or source files for building an executable.

Sirno treats entry artifact bytes as opaque lake state by default.
An artifact tree becomes a charm only when it contains a charm manifest
and the owning entry is enabled by local project configuration.
Artifact commands continue to manage bytes.
Charm commands interpret those bytes for setup, check, build, and cache maintenance.

A direct charm resolves an artifact script or executable into a spell.
A source charm builds a spell into Sirno cache state.
The charm remains the reviewed lake object.
The spell is the runtime object invoked by hooks.
