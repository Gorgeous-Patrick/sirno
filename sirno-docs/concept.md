---
name: Concept
description: A named idea that compresses project knowledge.
category:
  - meta
  - concept
belongs:
  - meta
---

A concept is a named idea that compresses project knowledge.

It may name a design intention, an algorithmic detail, local vocabulary,
a behavior, a test rationale, or the shared reason behind several decisions.

Concepts keep intent portable across the monograph, entries, and repo.
They let people and agents refer to a bundle of meaning without restating it.

The value of a concept is not that it is abstract.
The value is that it lets concrete work keep its reason attached.
A parser rule, a storage convention, a CLI behavior, and a test property may all share one design idea.
When that idea has a stable entry,
the project can refer to it instead of rediscovering it in every local context.

Concepts are also a scale control.
Some concepts are broad, such as the Sirno Lake or concept-driven development.
Other concepts are narrow, such as generated footer ownership or witness lookup.
Both are useful when they compress repeated understanding into a name.

The initialized `concept` entry is ordinary.
It is created by `init` and later operations do not privilege it.

That ordinariness is part of the design.
Projects may create their own categories and concepts without asking Sirno to recognize special cases.
Sirno supplies the structural fields and id rules.
The project supplies the vocabulary that makes its own design legible.

---

> **Sirno generated links begin. Do not edit this section.**

belongs (to):
- [meta](meta.md)

belongs (from): (none)

> **Sirno generated links end.**
