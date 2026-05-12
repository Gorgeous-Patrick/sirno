---
name: Witness
description: A structural marker from a design entry to repository evidence.
category:
  - concept
---

`witness:` declares that an entry's claim is evidenced in the repository.

The marker is canonical and has no value.
Sirno queries witnesses through `mosaika` by entry id.

The witness may be source code, tests, configuration, generated files, assets,
or any repository artifact that `mosaika` can mark and query.
A test may witness an entry when the test itself is the relevant code.

The entry body may explain how to find or interpret evidence as fallback guidance.
The structural convention remains the marker plus the entry id.

Witnesses connect prose to artifacts without merging the two.
The entry states the design claim in project language.
The repository mark identifies the artifact that should be inspected.
The entry id ties them together.

This marker is useful when a claim should be reviewable in code.
An implementation module can witness an interface decision.
A test can witness a behavioral property.
A configuration file can witness a storage or tool boundary.
A generated asset can witness a visible or packaged result.

The `witness:` marker should be used deliberately.
If an entry describes an idea that has no repository evidence yet,
leaving the marker absent is clearer.
If the evidence exists but is hard to interpret,
the entry body can explain what a reviewer should look for.
The structural key remains simple:
presence of `witness:` means the entry id is the query key.

---

> **Sirno generated links begin. Do not edit this section.**

- none

> **Sirno generated links end.**
