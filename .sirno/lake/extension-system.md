---
name: Charm And Spell System
desc: The route for entry-owned charms, resolved spells, and future hook invocation.
category:
  - concept
belongs:
  - entry-artifact
prerequisite:
  - entry-artifact
  - project-config
---

The charm and spell system lets a Sirno-managed project run user-owned code
from entry-owned artifacts.

This entry is the route through the implemented nucleus and the remaining hook proposal.
Its local entries own the terms and policies that can change independently.

The implemented nucleus is direct operator execution.
Sirno discovers charm manifests in entry artifact trees,
requires local enablement through `Sirno.toml`,
prepares and builds charms through `sirno charm`,
and inspects or runs resolved spells through `sirno spell`.

Hook-driven invocation remains proposal work.
Hook entries should later refine this route with their own trigger and result contracts.

The central split is between *charms* and *spells*.
A charm is an entry-owned artifact bundle.
A spell is the ready-to-run script or executable resolved from a charm.
Charms are prepared.
Spells are invoked.
Direct operator commands invoke spells today.
Hooks should invoke spells after the hook design is accepted.

Hook entries should refine this proposal.
Each hook entry should name the hook id, trigger point, event payload,
stdout contract, failure policy, and ordering constraints.
The charm and spell system supplies runnable artifacts and process execution.
The hook design supplies when and why spells run.

The current route is:

| Entry | Status | Design object |
|---|---|---|
| `charm` | Implemented | Entry-owned artifact bundle with reviewed design intent and runnable material. |
| `spell` | Implemented | Ready-to-run script or executable resolved from a charm. |
| `charm-manifest` | Implemented | `Sirno.charm.toml` and its command declaration rules. |
| `charm-resolution` | Implemented | Direct and source charm resolution, setup, check, build, and spell cache state. |
| `charm-enablement` | Implemented | `Sirno.toml` opt-in, trust, filesystem authority, and review policy. |
| `charm-and-spell-commands` | Implemented | `sirno charm` preparation and `sirno spell` invocation commands. |
| `spell-invocation` | Proposal | Hook event envelope, process output, failure handling, and ordering. |
