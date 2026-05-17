---
name: Witness Linking Discipline
desc: The agent procedure for linking lake entries to repository evidence.
category:
  - meta
belongs:
  - agent-skills
---

Witness linking connects an entry claim to the repository region that makes it inspectable.

Read the target entry before touching code.
Understand the claim, its structural fields, and any body guidance about what the evidence should
mean.
If no existing entry matches the need precisely,
create or propose a compact entry first and keep the witness id tied to that exact claim.
Reusing a near-enough entry id makes review less precise.

Choose the smallest region that supports the claim.
Inspect current witnesses with `sirno witness ENTRY_ID --full` before adding new ones.
Prefer a single item, test case, config stanza, or small cohesive block.
If a region is too broad, split it into smaller blocks that share the same entry id.
Place blocks inside configured repository members,
and use the configured delimiter syntax for the file kind.
Update the entry prose when needed so it briefly says what the region demonstrates,
and leave generated footers untouched.

Do not duplicate `mosaika`.
Sirno calls `mosaika` for delimiter matching, region extraction, and spans;
Sirno-side work consumes that structured output and formats it for review.

Validate after changing evidence.
Run the direct witness query again,
run the review-mode structural check,
and run render maintenance if lake metadata or links changed.
Then read the full witness output as a human would:
it should show concise ranges, the literal matched region, and no broad unrelated code.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills](agent-skills.md)
- belongs (from): (none)

> **Sirno generated links end.**
