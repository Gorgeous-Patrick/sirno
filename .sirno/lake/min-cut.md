---
name: Min-Cut
desc: A documentation principle that keeps canonical project truth small, unique, and self-contained.
category:
  - meta
  - concept
belongs:
  - methodology
prerequisite:
  - methodology
  - entry
  - semantic-locality
refines:
  - documentation-driven-development
  - semantic-locality
---

Min-cut is the documentation principle that keeps a project's canonical truth small,
unique, and self-contained.

A project gives each durable design fact one authoritative home.
In a Sirno-managed project, that home is usually one *entry*.
The entry should be compact enough to review,
complete enough to trust,
and local enough for a newcomer to understand before following links.

The canonical surface is the maintenance boundary.
A smaller boundary lowers the number of places a reviewer must inspect
and lowers the number of places an edit must keep aligned.
When the source is small and unique,
newcomers can learn where to look
and maintainers can see when a change has touched the whole claim.

Routes, projections, translations, examples, tutorials, and generated views
may reshape material for a reader.
Their job is to carry people toward the maintained source
and make that source easier to use.

Min-cut works with semantic locality.
Semantic locality asks each *entry* to make sense in place.
Min-cut asks the project to keep the set of authoritative places small enough
that the *lake* remains teachable and reviewable.

A practical check:

- Does each durable fact have one authoritative *entry*?
- Can a newcomer find the maintained source from the first narrative route?
- Can a reviewer inspect the relevant canonical surface in one focused pass?
- Are derived materials tied back to the *entry* they explain?
- Does the *entry* carry enough meaning to outlive the edit that introduced it?
