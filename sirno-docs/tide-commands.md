---
name: Tide Commands
desc: Commands and MCP tools for dependency review workitems.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

Tide commands expose dependency review obligations created by structural changes.

`sirno tide status` reports entry ids that need dependency review,
grouped by review entry in one table.
The reason column lists the ripple entry whose change created the review obligation.
It prints a one-sentence summary after the table.

`sirno tide status --by wave` groups the same output by wave.
`sirno tide status --show full` reports open dependency review obligations
in the same grouped table.
`sirno tide status --show all` also reports resolved obligations.
`sirno tide status --by entry` selects the default review-entry grouping explicitly.
`sirno tide status -o, --format` selects `human` or `json` output.

The canonical review command forms are `sirno tide resolve` and `sirno tide unresolve`.
The top-level forms `sirno resolve` and `sirno unresolve` select the same operations.

`sirno resolve ENTRY_ID` resolves open workitems whose neighbor is that entry.
`sirno resolve RIPPLE,FIELD,DIRECTION,NEIGHBOR` resolves one full workitem tuple.
`sirno resolve --infer` resolves open workitems whose neighbor also appears in the ripple set.
`sirno resolve --json JSON` resolves full workitem tuples encoded as JSON.

`sirno unresolve ENTRY_ID` removes resolutions whose neighbor is that entry.
`sirno unresolve RIPPLE,FIELD,DIRECTION,NEIGHBOR` removes one full workitem resolution.
`sirno reopen` is an alias for `sirno unresolve`.
`sirno tide reopen` is an alias for `sirno tide unresolve`.
`sirno tide reset` clears tide resolution state.

MCP tide tools use typed selectors.
`sirno_tide_status` returns review entry ids by default.
Its `show` argument selects `review`, `full`, or `all`.
`sirno_tide_resolve` and `sirno_tide_unresolve`
accept neighbor id arrays and existing JSON-shaped workitem objects.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
