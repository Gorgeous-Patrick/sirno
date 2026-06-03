---
name: Spec-Driven Development
desc: A development style that starts from explicit behavioral commitments.
category:
  - concept
belongs:
  - development-style
prerequisite:
  - concept-driven-development
refines:
  - concept-driven-development
---

Spec-driven development starts from explicit behavioral commitments.

A specification says what must be true before code chooses how to make it true.
It can describe an interface, state transition, invariant, file shape, command contract,
or observable behavior.
The useful part is precision:
the work has a named claim that implementation can satisfy, revise, or reject.

In concept-driven development,
a specification sharpens a concept.
The concept gives the specification a stable place in the project vocabulary.
The specification gives the concept a local behavioral edge.

Spec-driven work should stay small enough to test or inspect.
If a specification becomes a broad essay,
split it into smaller *entries* or refine it under the concept it makes concrete.
If implementation reveals a better boundary,
internalize that boundary into the *lake* so the specification remains a living design object.
