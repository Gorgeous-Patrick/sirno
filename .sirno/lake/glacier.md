---
name: Glacier
desc: A managed lakelet formed by crystallizing an upstream lake.
category:
  - concept
belongs:
  - lake-composition
  - lake-namespace
  - sirno-upstream
prerequisite:
  - upstream-lake
  - lakelet
  - crystallization
  - entry-freeze
---

A *glacier* is a managed lakelet formed by crystallizing an upstream lake.

A glacier occupies the declared glacier domain in the current lake.
For example,
crystallizing upstream entry `design` under domain `core` writes the glacier entry
as `lake/core/design.md` with entry address `core.design`.

Glacier entries carry the `managed` frozen reason.
Crystallization owns adding, replacing, and removing that managed content.
Project-managed lakelet files cannot occupy the same glacier domain
unless they are already managed by crystallization.
