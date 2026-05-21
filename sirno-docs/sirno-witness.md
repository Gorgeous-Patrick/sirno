---
name: Witness
desc: Repository evidence for a design entry.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - entry
  - repo
---

A *witness* is *repository* evidence for an *entry* claim.

A *witness* is a validation link, but NOT a storage mechanism.
The *entry* holds the readable form of the design:
prose, code, diagrams, configuration, or fixtures, whichever reads clearest.
Including code or configuration inside an *entry* is always acceptable.
The *witness* block names a *repository* region whose behavior should be inspectable
against the *entry*'s claim,
so a reviewer can confirm that the running code does what the design says it should.
Reaching for a *witness* block to save information is the wrong tool.

The *witness* entry is the front door for *repository* evidence.
Its review neighborhood covers how blocks are marked,
how they are looked up and renamed,
how resolved records are shaped,
and how malformed blocks are reported.

Sirno discovers *witness* status mechanically.
It queries *witnesses* through `mosaika` by *entry* id.
Humans should run `sirno witness ENTRY_ID --full` to inspect evidence.
Agents should use the corresponding MCP witness tool instead of inferring evidence
from prose or generated links.

The *witness* may be source code, tests, configuration, generated files, assets,
or any *repository* artifact that `mosaika` can delimit and query.
A test may witness an *entry* when the test itself is the relevant code.

A *repository witness* block opens with `sirno:witness:<entry-id>:begin`
and closes with `sirno:witness:<entry-id>:end`.
The opening and closing *entry* ids must match.
The *witness delimiter* defines the configured marker shapes and capture rules,
and *witness lookup* defines the scan over `[repo].members`
and the rename rewrites that follow `sirno entry rename`.

The *entry* body may explain how to find or interpret evidence as fallback guidance.
The convention is the *entry* id plus the *repository witness* block.

Repository *witnesses* connect prose to artifacts without merging the two.
The *entry* states the design claim in project language.
The *witness* block identifies the artifact region that should be inspected.
The *entry* id ties them together.

Evidence is useful when a claim should be reviewable in the repo.
An implementation module can witness an interface decision.
A test can witness a behavioral property.
A configuration file can witness a storage or tool boundary.
A generated asset can witness a visible or packaged result.

When *repository* evidence supports a related but different claim,
create a new *entry* and witness that exact claim.
Reusing a near-enough *entry* id makes review less precise.

If an *entry* describes an idea that has no *repository* evidence yet,
leaving it unwitnessed is clearer.
If the evidence exists but is hard to interpret,
the *entry* body can explain what a reviewer should look for.
The *entry* id remains the query key.

How Sirno represents resolved evidence is the *witness record*.
How malformed evidence is reported is *witness integrity*.
This *entry* stays the front door;
its neighborhood carries the precise mechanics.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [witness-delimiter](witness-delimiter.md)
  - [witness-integrity](witness-integrity.md)
  - [witness-lookup](witness-lookup.md)
  - [witness-record](witness-record.md)

> **Sirno generated links end.**
