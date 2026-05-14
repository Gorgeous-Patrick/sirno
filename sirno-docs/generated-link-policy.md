---
name: Generated Link Policy
description: Configuration that chooses which structural links appear in generated footers.
category:
  - concept
refines:
  - generated-footer
---

Generated link policy decides which metadata-derived sections appear in a generated footer.

`category`, `belongs`, and `refines` can each generate outgoing links to targets
and incoming links from sources.
A boolean setting enables or disables both link sides.
A `{ to = ..., from = ... }` setting chooses the two link sides separately.

`links.clique` adds separate clique-derived sections.
It does not change direct belongs projection.
When enabled, each `belongs` target induces clique edges:
the target links to its members,
and members link to the target and to one another.
When disabled, only configured structural field sections are rendered.

This policy is configuration, not entry data.
Changing it alters generated navigation surfaces without changing structural metadata.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to): (none)

> **Sirno generated links end.**
