---
name: Form
description: A shape project knowledge takes inside Sirno.
category:
  - concept
  - narrative
belongs:
  - sirno
---

Sirno works through three forms.

`mono` is one optional configured Markdown document.
It carries the whole project design as a readable monograph.

`sirno` is one configured entry lake.
It contains compact Markdown entries with exact metadata.
When Sirno Frost is configured, it is versioned through a separate `eter` Frost root,
so one lake version names one immutable entry set.

`repo` is the repository artifact form.
It contains source, tests, configuration, generated files, assets,
and any artifact that can realize or witness design.
Sirno scans repository witnesses only when `[repo].members` is configured.

The forms are not just storage locations.
They are roles in a design workflow.
The monograph is optimized for continuity,
so a reader can build a mental model in a deliberate order.
The lake is optimized for addressability,
so a person or tool can find the named object that matters to a local change.
The repo form is optimized for execution and evidence,
so design commitments have concrete artifacts to inspect.

Before the lake exists, the user chooses whether the repo or monograph carries more authority.
Once the lake is established, Sirno treats it as the structured intermediate form.

That authority can still be revised by deliberate work.
Lowering lets a monograph seed the lake.
Reflecting lets implementation discoveries update the lake.
Raising lets the lake rebuild a whole-project narrative.
Realizing lets entries guide implementation.

Keeping the three forms distinct prevents one document from trying to serve every reader at once.
The monograph can stay fluent.
Entries can stay compact and named.
Repository artifacts can stay focused on behavior while still having a place to point for intent.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to):
- [sirno](sirno.md)

> **Sirno generated links end.**
