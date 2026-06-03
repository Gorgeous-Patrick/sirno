---
name: Test-Driven Development
desc: A development style that lets tests state and protect expected behavior.
category:
  - concept
belongs:
  - development-style
prerequisite:
  - concept-driven-development
refines:
  - concept-driven-development
---

Test-driven development starts from executable expectations.

A test names behavior before or alongside the implementation that should satisfy it.
It turns uncertainty into a small feedback loop:
state the expected property,
make the code answer it,
then keep the test as a guard for future changes.

In concept-driven development,
a test protects a concept by making one of its claims executable.
The concept explains why the behavior matters.
The test shows when that behavior holds.

Test-driven work is strongest when tests carry design intent,
not only regression coverage.
A good test should help a reviewer see the property being preserved.
When a test exposes a clearer concept or a narrower invariant,
internalize that discovery into the *lake* instead of leaving it trapped in test code.
