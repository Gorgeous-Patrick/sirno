---
desc: A Git-backed Sirno project declared by the current project.
name: Upstream Lake
category:
  - concept
belongs:
  - lake-composition
  - upstream
prerequisite:
  - project-config
  - upstream-file
  - lake-namespace
---

An *upstream lake* is a Git-backed Sirno project declared by the current project.

The current project names each upstream in `Sirno.toml`.
The declaration chooses a Git source,
one ref selector,
an entry-domain atom for the glacier address prefix,
an optional project root inside the Git tree,
an optional project config manifest path,
and optionally a mist name from the upstream project.
That domain atom is always explicit.
Sirno does not infer a local upstream name from the Git source.
The source may be a remote Git URL or a local Git repository source accepted by Git.
Local repository sources are read through committed Git objects;
dirty worktree state is ignored.
The project root defaults to `.`.
The manifest path is relative to that root and defaults to `Sirno.toml`.
The manifest may be nested or named differently.

Every upstream is explicitly included by crystallizing it into a glacier.
Sirno does not have a path-only upstream,
and it does not leave declared upstreams as linked but unexpanded dependencies.
When the declaration names a mist,
crystallization applies that upstream mist before rebasing entry addresses into the glacier domain.
Only selected upstream entries and their artifacts are imported.
Intrinsic and structural metadata-definition entries are also imported,
so imported field keys can resolve to definitions inside the glacier.
An upstream with no mist imports the complete upstream lake.

Upstream fetches are cached in the global Sirno store under `~/.sirno`.
The cache stores one Git mirror for each normalized upstream URI.
Projects reuse those mirrors across lake systems.
