---
name: Direction
description: A concept that names movement between Sirno surfaces.
category:
  - concept
---

A direction names work between surfaces.

Sirno uses four direction names:
`lower`, `raise`, `realize`, and `reflect`.

Their direct names are also useful:
`mono-to-sirno`, `sirno-to-mono`, `sirno-to-code`, and `code-to-sirno`.

Directions are vocabulary for humans, LLMs, skills, CLI surfaces, and MCP tools.
They describe coherent work without requiring every direction to be a one-shot command.

The direction names make design work easier to request and review.
Instead of saying "split this document into smaller pieces" every time,
a user can ask to lower a monograph into the store.
Instead of saying "update the design notes based on this code change",
a user can ask to reflect the code into entries.

The four directions form a loop:
`mono` lowers into `sirno`,
`sirno` realizes into `code`,
`code` reflects back into `sirno`,
and `sirno` raises into `mono`.
The loop is conceptual, not automatic authority.
Each movement should still be performed with judgment about the current source of truth.

This vocabulary also helps skills stay focused.
A lowering skill should preserve narrative intent while creating entries.
A realization skill should inspect entries before editing code.
A reflection skill should record durable design facts learned from implementation.
A raising skill should compose a readable monograph.

---

> **Sirno generated links begin. Do not edit this section.**

Category (from): (none)

Category (to)
- [concept](concept.md)

Clique
- [lower](lower.md)
- [raise](raise.md)
- [realize](realize.md)
- [reflect](reflect.md)

> **Sirno generated links end.**
