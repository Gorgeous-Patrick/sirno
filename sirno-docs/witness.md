---
name: Witness
desc: Repository evidence for a design entry.
category:
  - concept
belongs:
  - sirno
---

A *witness* is repository evidence for an *entry* claim.

The *witness* entry is the front door for repository evidence.
Its local *entries* cover lookup behavior and the *repository* surface where *witnesses* live.

Sirno discovers *witness* status mechanically.
It queries *witnesses* through `mosaika` by *entry* id.
Agents should use `sirno witness ENTRY_ID --full` to inspect evidence instead of inferring it
from prose or generated links.

The *witness* may be source code, tests, configuration, generated files, assets,
or any repository artifact that `mosaika` can delimit and query.
A test may witness an *entry* when the test itself is the relevant code.

Repository artifacts are selected by `[repo].members`.
Directory members are scanned recursively.
The repository *witness* block opens with `sirno:witness:<entry-id>:begin`
and closes with `sirno:witness:<entry-id>:end`.
The opening and closing *entry* ids must match.
Rust and other line-commented files can write the sentinels with `//`.
Markdown files can write them as hidden HTML comments.
This is the standard syntax written by generated configs.
The standard delimiter regex uses one canonical capture for filename-like *entry* ids.
It captures all legal ids that can fit between the sentinel colons.
The parsed *entry* id then applies the remaining cross-platform filename checks.
A project can override the delimiter regex pairs with `[[witness.delimiters]]`
when another repository surface needs a different marker shape.

The *entry* body may explain how to find or interpret evidence as fallback guidance.
The convention is the *entry* id plus the repository *witness* block.

Repository *witnesses* connect prose to artifacts without merging the two.
The *entry* states the design claim in project language.
The *witness* block identifies the artifact region that should be inspected.
The *entry* id ties them together.

Evidence is useful when a claim should be reviewable in the repo.
An implementation module can witness an interface decision.
A test can witness a behavioral property.
A configuration file can witness a storage or tool boundary.
A generated asset can witness a visible or packaged result.

When repository evidence supports a related but different claim,
create a new *entry* and witness that exact claim.
Reusing a near-enough *entry* id makes review less precise.

If an *entry* describes an idea that has no repository evidence yet,
leaving it unwitnessed is clearer.
If the evidence exists but is hard to interpret,
the *entry* body can explain what a reviewer should look for.
The *entry* id remains the query key.

The repository *witness* for this *entry* should show how Sirno represents *witness* records,
spans,
and accepted delimiter styles after `mosaika` finds the delimited repository regions.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [repo](repo.md)
  - [witness-fixture-isolation](witness-fixture-isolation.md)

> **Sirno generated links end.**
