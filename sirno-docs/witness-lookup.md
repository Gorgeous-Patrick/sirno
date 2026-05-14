---
name: Witness Lookup
description: The mosaika-backed scan that resolves witness blocks by entry id.
category:
  - concept
refines:
  - witness
---

Witness lookup resolves repository evidence by scanning configured repo members with `mosaika`.

`[repo].members` defines the repository artifact surface when witness lookup is configured.
File members are scanned directly.
Directory members are scanned recursively.
Glob members expand to matching files.

Sirno projects each member file into a `mosaika` transform that logs witness blocks.
The opening and closing delimiters both capture the entry id.
Sirno rejects a witness block when the delimiter ids differ.
Sirno parses the log stream into witness records keyed by entry id.
The stored delimiter spans exclude leading indentation.
Full output displays every line spanned by the matched block,
then dedents that line set for review.

The lookup path keeps witness syntax out of entry prose.
Entries remain design claims.
Repository artifacts carry the precise source spans that witness those claims.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to): (none)

> **Sirno generated links end.**
