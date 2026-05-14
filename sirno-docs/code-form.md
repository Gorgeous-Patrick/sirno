---
name: Code Form
description: The repository artifacts that realize and witness design entries.
category:
  - concept
clustee:
  - form
  - witness
refiner:
  - form
witness:
---

The code form is the repository implementation form.

It includes source files, tests, configuration, generated files, assets,
and other artifacts that realize design decisions.

Repository artifacts can witness entries through `mosaika`.
Sirno uses the entry id as the witness query key,
keeping design names and witness blocks connected without embedding block syntax in entry prose.
`[code].members` defines the repository artifact surface that Sirno scans when configured.
File members are scanned directly,
and directory members are scanned recursively.
Witness blocks open with `sirno:witness:<entry-id>:begin`
and close with `sirno:witness:<entry-id>:end`.
Both sentinels name the same entry id.

The code form is where design becomes costly in the useful sense.
Names, invariants, parser choices, storage boundaries, user interfaces,
tests, and generated assets all make commitments that future work must honor or revise.
Sirno does not ask every line of code to carry a design entry.
It asks important commitments to have a name that can survive beyond the edit that introduced them.

Witnesses make that name concrete.
An entry can state a claim,
and witness blocks can show where the claim is implemented, tested, configured, or generated.
The witness block belongs to the repository artifact.
The entry keeps the design language.
The shared key is the entry id.

This keeps code and documentation coupled without making either one awkward.
Code does not need long narrative comments for every design concept.
Entries do not need to duplicate source snippets that will drift.
Review can move between them by asking which entry explains a code commitment,
and which repository artifact witnesses an entry.

---

> **Sirno generated links begin. Do not edit this section.**

Clustee (to)
- [form](form.md)
- [witness](witness.md)

> **Sirno generated links end.**
