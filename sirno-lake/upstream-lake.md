---
name: Upstream Lake
desc: A Git-backed Sirno project declared by the current project.
category:
  - concept
belongs:
  - sirno-upstream
prerequisite:
  - project-config
  - sirno-lock
  - entry-domain
---

An *upstream lake* is a Git-backed Sirno project declared by the current project.

The current project names each upstream in `Sirno.toml`.
The declaration chooses a Git source,
one ref selector,
and an entry-domain atom for the glacier address prefix.
That domain atom is always explicit.
Sirno does not infer a local upstream name from the Git source.
The source may be a remote Git URL or a local Git repository source accepted by Git.
Local repository sources are read through committed Git objects;
dirty worktree state is ignored.

Every upstream is explicitly included by crystallizing it into a glacier.
Sirno does not have a path-only upstream,
and it does not leave declared upstreams as linked but unexpanded dependencies.

Upstream fetches are cached in the global Sirno store under `~/.sirno`.
The cache stores one Git mirror for each normalized upstream URI.
Projects reuse those mirrors across lake systems.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-upstream](sirno-upstream.md)
- belongs (from):
  - [sirno-lock](sirno-lock.md)

> **Sirno generated links end.**
