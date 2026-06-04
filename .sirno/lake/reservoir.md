---
desc: The canonical tracked lake store kept under .sirno/lake.
name: Reservoir
category:
  - concept
  - implemented
belongs:
  - lake
prerequisite:
  - lake
refines:
  - lake
---

A *reservoir* is the canonical Sirno Lake storage surface.
It refines the Sirno Lake by choosing the tracked directory that stores authored lake content.

The default reservoir path is `.sirno/lake` next to `Sirno.toml`.
It holds the tracked Markdown *entries* and lake-owned *entry artifacts*
that Anchor, Tide, query, upstream crystallization, and Git treat as canonical.

The reservoir is quiet storage.
It is not the default place for humans or agents to browse, edit, or receive rendered navigation.
Those working surfaces are *misty lakes* produced by a *mist*.

The reservoir keeps design authority separate from local projection shape.
Git versions the reservoir and Sirno control files.
A *misty lake* may be untracked workspace material,
but intake must write accepted mist ripples back into the reservoir before Anchor accepts them.

The reservoir stores authored entry content.
Rendered output belongs in *misty lakes*.
This keeps generated navigation, local filters, editor state, and agent workspaces
from becoming canonical storage concerns.

The *repository witnesses* for this entry should show the entry directory root,
check settings, report shape, and write path that treat the reservoir as parsed lake storage.
