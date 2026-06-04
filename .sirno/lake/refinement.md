---
name: Refinement
desc: A discipline for making concrete work preserve broader design claims.
category:
  - concept
  - meta
belongs:
  - methodology
prerequisite:
  - concept
---

Refinement is the discipline of making a concrete version of a broader claim
while preserving the observable promise that made the broader claim useful.

Formal methods use refinement to relate an abstract specification to a concrete model,
program, or implementation.
The concrete artifact may add variables, cases, representation detail, or strategy.
It remains a refinement only when each concrete behavior can be read as an allowed
abstract behavior through an explicit relation.
That relation may be a refinement mapping, a retrieve relation, a gluing invariant,
or a set of proof obligations.

A project can use refinement without adopting a formal notation.
Sirno uses it as a practical design discipline.
It does not require formal proof,
but it keeps the same obligation in ordinary project language.
A refined *entry* should say what broader claim it makes concrete,
which new detail it introduces,
and what must still be preserved.
The relation is strongest when a reviewer can move from the concrete *entry*
back to the abstraction without losing the promise being carried.

Refinement is not decomposition by outline.
A smaller file, paragraph, or task is not a refinement just because it sits underneath
something larger.
A refinement introduces detail under obligation:
the new detail should still satisfy the higher-level claim.
When the detail changes the claim,
internalize the changed claim in the broader *entry* instead of hiding it below.

Use refinement when work crosses abstraction levels.
A concept can be refined into an invariant, storage rule, command contract,
parser behavior, test, or code region.
State the preservation obligation in prose before relying on structure alone.
Add *repository witnesses* when the repository contains evidence for the refined claim.
If the relation from concrete detail back to abstract promise is hard to explain,
the design may need a smaller *entry*, a better name, or an explicit bridge concept.

A healthy refinement chain stays reviewable.
Each step should let a reviewer ask three questions:
what was made more concrete, what relation carries it upward,
and what evidence would show that the relation still holds?
The chain may stop in prose, tests, or implementation.
It should not bury an important design change at the bottom of the chain.
