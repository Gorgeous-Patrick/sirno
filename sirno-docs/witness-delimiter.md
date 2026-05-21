---
name: Witness Delimiter
desc: The configured begin/end regex pair that marks a witness block by entry address.
category:
  - concept
belongs:
  - sirno-witness
prerequisite:
  - sirno-witness
---

A *witness delimiter* is a configured regex pair that marks a block.
`[[witness.delimiters]]` lists the pairs.
Each table has a `begin` and an `end` pattern,
and both patterns capture the *entry address* in their first capture group.

The standard syntax is what generated configs write.
It accepts `//` line comments for Rust and similar files
and hidden HTML comments for Markdown.
Both standard patterns share one canonical capture
that admits every valid *entry address* allowed between the sentinel colons.
The captured text is then parsed through the normal *entry address* checks.

A project may add `[[witness.delimiters]]` pairs
when another repository surface needs a different marker shape.
The configured pairs are the only delimiters Sirno scans for.
An empty delimiter list disables repository witness lookup.

Each configured pattern is validated before use.
A pattern that is not a valid regex is rejected.
A pattern whose first group does not capture the *entry address* is rejected.
A pattern that can match empty text is rejected.
These checks keep the scan well-defined,
so every matched block yields a definite id and region.

The delimiter is where prose and evidence meet without mixing.
The *entry address* inside the delimiter is the link.
Block contents stay owned by the repository artifact.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-witness](sirno-witness.md)
- belongs (from): (none)

> **Sirno generated links end.**
