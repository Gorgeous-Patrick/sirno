---
name: Charm And Spell System
desc: A proposal route for executable entry artifacts invoked from Sirno hook points.
category:
  - concept
  - proposal
belongs:
  - future-work
  - entry-artifact
prerequisite:
  - entry-artifact
  - project-config
refines:
  - future-work
---

The charm and spell system lets a Sirno-managed project run user-owned code
from Sirno hook points.

This entry is the proposal route.
Its local entries own the terms and policies that can change independently.

The central split is between *charms* and *spells*.
A charm is an entry-owned artifact bundle.
A spell is the ready-to-run script or executable resolved from a charm.
Charms are prepared.
Spells are invoked.
Hooks invoke spells.

Hook entries should refine this proposal.
Each hook entry should name the hook id, trigger point, event payload,
stdout contract, failure policy, and ordering constraints.
The charm and spell system supplies runnable artifacts and process execution.
The hook design supplies when and why spells run.

The current route is:

| Entry | Design object |
|---|---|
| `charm` | Entry-owned artifact bundle with reviewed design intent and runnable material. |
| `spell` | Ready-to-run script or executable resolved from a charm. |
| `charm-manifest` | `Sirno.charm.toml` and its command declaration rules. |
| `charm-resolution` | Direct and source charm resolution, setup, check, build, and spell cache state. |
| `charm-enablement` | `Sirno.toml` opt-in, trust, filesystem authority, and review policy. |
| `spell-invocation` | Hook event envelope, process output, failure handling, and ordering. |
| `charm-and-spell-commands` | `sirno charm` preparation commands and `sirno spell` invocation commands. |
