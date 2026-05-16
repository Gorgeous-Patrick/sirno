---
name: Concept-Driven Development
desc: A working style that uses named concepts to compress project knowledge.
category:
  - concept
  - narrative
belongs:
  - sirno
refines:
  - concept
---

Concept-driven development starts from compression.

A concept gives a stable name to knowledge that would otherwise be repeated:
specifications, decisions, implementation notes, tests,
and the shared reason behind them.

The LZ77 analogy is an adaptive dictionary.
Sirno gives project knowledge a similar dictionary,
except each reference remains human-readable.

Concepts serve three roles at once.
They cluster behavioral specifications under one named object.
They keep intent portable across levels of detail.
They organize tests so properties and constraints become easier to see.

Concept-driven development concludes the other three practices by asking what named idea
their work should preserve.
Spec-driven development says what must hold.
Intent-driven development says why the work matters.
Test-driven development makes one claim executable.
The concept gathers those pressures into a durable project object
that future work can inspect, refine, realize, and witness.

The method is practical.
Before local work begins,
the project asks which named idea explains the work.
If the answer is missing or vague,
the design should be lowered or reflected until the idea has an entry.
That entry does not need to solve the whole problem.
It only needs to give the work a stable handle and enough prose to keep the reason visible.

Concept-driven development does not replace specifications, intent, or tests.
It gives those practices a shared conclusion.
A specification can point to the concept it sharpens.
Intent can name the concept it is trying to keep alive.
A test can witness the concept it protects.
Implementation can refine a concept when code reveals a clearer representation.

This style helps agents as much as humans.
An agent with limited context can inspect the concept and its structural fields,
then work locally without carrying the whole monograph.
The concept becomes a compact bridge between broad intent and detailed code.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from): (none)

> **Sirno generated links end.**
