---
name: Witness
description: A structural marker from a design entry to repository evidence.
category:
  - concept
clustee:
  - sirno
  - structural-field
witness:
---

`witness:` declares that an entry's claim is evidenced in the repository.

The witness entry is the module closure for repository evidence.
Its member entries cover lookup behavior and the code artifact surface where witnesses live.

The marker is canonical and has no value.
Sirno queries witnesses through `mosaika` by entry id.

The witness may be source code, tests, configuration, generated files, assets,
or any repository artifact that `mosaika` can delimit and query.
A test may witness an entry when the test itself is the relevant code.

Repository artifacts are selected by `[code].members`.
Directory members are scanned recursively.
The repository witness block opens with `sirno:witness:<entry-id>:begin`
and closes with `sirno:witness:<entry-id>:end`.
The opening and closing entry ids must match.

The entry body may explain how to find or interpret evidence as fallback guidance.
The structural convention remains the metadata marker plus the witness block.

Witnesses connect prose to artifacts without merging the two.
The entry states the design claim in project language.
The witness block identifies the artifact region that should be inspected.
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

Clustee (to)
- [sirno](sirno.md)
- [structural-field](structural-field.md)

> **Sirno generated links end.**
