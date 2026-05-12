---
name: Concept-Driven Development
description: A working style that uses named concepts to compress project knowledge.
category:
  - concept
clustee:
  - sirno
refiner:
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

The method is practical.
Before local work begins,
the project asks which named idea explains the work.
If the answer is missing or vague,
the design should be lowered or reflected until the idea has an entry.
That entry does not need to solve the whole problem.
It only needs to give the work a stable handle and enough prose to keep the reason visible.

Concept-driven development is not a rejection of specifications, tests, or implementation-first learning.
It gives those practices a shared dictionary.
A specification can point to the concept it sharpens.
A test can witness the concept it protects.
Implementation can refine a concept when code reveals a clearer representation.

This style helps agents as much as humans.
An agent with limited context can inspect the concept and its structural fields,
then work locally without carrying the whole monograph.
The concept becomes a compact bridge between broad intent and detailed code.

---

> **Sirno generated links begin. Do not edit this section.**

- [sirno](sirno.md)

> **Sirno generated links end.**
