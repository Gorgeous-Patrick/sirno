---
name: Witness Integrity
desc: The check-time failures for mismatched, orphan, or invalid witness delimiters.
category:
  - concept
belongs:
  - sirno-witness
prerequisite:
  - witness-lookup
---

*Witness integrity* is the set of failures Sirno reports
when a *repository witness* block is malformed.
It is the *witness* side of structural checking.
A clean scan yields only well-formed *witness records*.

A block fails when its opening and closing ids differ.
`sirno:witness:a:begin` paired with `sirno:witness:b:end` is rejected,
because the pair no longer names one claim.

A delimiter fails when it has no partner.
The checker also scans each configured delimiter pattern as a single token.
A begin or end token that no resolved block consumed is an orphan delimiter.
The diagnostic names the file, the captured id, and the line and column,
and states which side is missing.

A marker fails when its captured text is not a valid *entry address*.
The text between the sentinel colons must pass the normal id checks,
or the block cannot be keyed to an *entry*.

These failures keep the *entry address* a reliable query key.
Evidence that cannot be tied to exactly one *entry*
is reported rather than silently indexed.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-witness](sirno-witness.md)
- belongs (from): (none)

> **Sirno generated links end.**
