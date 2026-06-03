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

`sirno tide status` reports entry addresses that need dependency review,
grouped by review entry in one table.
The reason column lists the ripple entry whose change created the review obligation.
It prints a one-sentence summary after the table.

`sirno tide status --by wave` groups the same output by wave.
`sirno tide status --show full` reports open dependency review obligations
in the same grouped table.
`sirno tide status --show all` also reports resolved obligations.
`sirno tide status --by entry` selects the default review-entry grouping explicitly.
`sirno tide status -o, --format` selects `human` or `json` output.

`sirno tide` and `sirno tide tui` open an interactive tide resolution UI.
It shows a selectable tide table, a selected-row detail panel, and a key/message footer.
The default view groups rows by review entry with open and resolved counts
and the ripple entries that caused review.
Tab toggles between review-entry and wave grouping.
`f` toggles between summary rows and full workitem rows.
`j`, `k`, Up, and Down move the selected row.
Space resolves the selected row.
`u` reopens the selected row.
In review-entry summary mode, these keys apply to the selected review entry.
In wave summary mode, they apply to exact workitems in the selected wave.
In full mode, they apply to the selected exact workitem.
`i` runs infer resolution.
`c` refreshes tide state.
`q` and Esc exit.

The canonical review command forms are `sirno tide resolve` and `sirno tide unresolve`.
The top-level forms `sirno resolve` and `sirno unresolve` select the same operations.

`sirno resolve ENTRY_ADDRESS` resolves open workitems whose neighbor is that entry.
`sirno resolve RIPPLE,FIELD,DIRECTION,NEIGHBOR` resolves one full workitem tuple.
`sirno resolve --infer` resolves open workitems whose neighbor also appears in the ripple set.
`sirno resolve --json JSON` resolves full workitem tuples encoded as JSON.

`sirno unresolve ENTRY_ADDRESS` removes resolutions whose neighbor is that entry.
`sirno unresolve RIPPLE,FIELD,DIRECTION,NEIGHBOR` removes one full workitem resolution.
`sirno reopen` is an alias for `sirno unresolve`.
`sirno tide reopen` is an alias for `sirno tide unresolve`.
`sirno tide reset` clears tide resolution state.

MCP tide tools use typed selectors.
`sirno_tide_status` returns review entry addresses by default.
Its `show` argument selects `review`, `full`, or `all`.
`sirno_tide_resolve` and `sirno_tide_unresolve`
accept neighbor path arrays and existing JSON-shaped workitem objects.
