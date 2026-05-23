---
name: Crystallization
desc: The operation that materializes upstream lakes as glaciers.
category:
  - concept
belongs:
  - sirno-upstream
prerequisite:
  - upstream-lake
  - entry-freeze
  - sirno-lock
---

*Crystallization* materializes declared upstream lakes into glaciers in the current lake.

For each selected upstream,
Sirno resolves the locked Git commit,
reads the upstream project's configured lake,
and writes its entries as a glacier under the declared domain.
For example, upstream entry `design` under domain `core` becomes `core.design`
and is written to `lake/core/design.md`.

Glacier artifacts are written under `.artifacts/<domain>.<entry-address>/...`.
Structural metadata targets inside the upstream lake are rebased through the glacier domain.
Generated footer regions are stripped during import
and regenerated for the current lake view.
Unknown leading-dot roots stay reserved and are not imported as entry domains.
The glacier domain must be empty or already managed by crystallization.
Unmanaged local lakelet files block crystallization for the same domain.

Glacier entries carry the `managed` frozen reason.
If the upstream entry already carried `reviewed`,
the glacier entry carries both `reviewed` and `managed`.
Normal melt removes only `reviewed`.
Crystallization owns adding and removing `managed`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-upstream](sirno-upstream.md)
- belongs (from): (none)

> **Sirno generated links end.**
