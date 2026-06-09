---
desc: A managed lakelet formed by crystallizing an upstream lake.
name: Glacier
category:
  - concept
belongs:
  - lake-composition
  - lake-namespace
  - upstream
prerequisite:
  - upstream-lake
  - lakelet
  - crystallization
  - entry-freeze
---

A *glacier* is a managed lakelet formed by crystallizing an upstream lake.

A glacier occupies the declared glacier domain in the current lake.
That domain supplies the rebased entry-address prefix for imported upstream entries.

Glacier entries carry the `managed` frozen reason.
Crystallization owns adding, replacing, and removing that managed content.
Project-managed lakelet files cannot occupy the same glacier domain
unless they are already managed by crystallization.
