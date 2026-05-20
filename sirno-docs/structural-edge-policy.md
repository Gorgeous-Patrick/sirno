---
name: Structural Edge Policy
desc: Configuration that chooses rendered links and tide review neighbors for structural edges.
category:
  - concept
belongs:
  - generated-navigation
prerequisite:
  - structural-field
---

Structural edge policy decides how configured structural edges participate in Sirno tooling.

`[structural]` lists the metadata fields Sirno treats as structural.
Each metadata field is configured by a `[structural.FIELD]` subtable.
The subtable may define `to`, `from`, and `clique` edge policies.
Each edge policy may enable `render`,
`ripple.lake`,
and `ripple.frost`.
Absent values are false.

`render = true` includes that edge direction in the generated footer.
The footer guard text still says generated links,
because it names the owned Markdown region rather than the command name.

`ripple.lake = true` adds waterline neighbors to the *tide*.
`ripple.frost = true` adds frostline neighbors to the *tide*.
There is no `ripple = true` shorthand.

```toml
[structural.belongs]
to = { ripple = { lake = true, frost = false }, render = true }
from = { ripple = { lake = true, frost = true }, render = true }
clique = { ripple = { lake = true } }

[structural.refines]
to = { ripple = { lake = true, frost = false } }
from = { ripple = { lake = true, frost = true } }
```

`to` follows outgoing metadata targets.
`from` follows incoming sources that name the current *entry*.
`clique` follows entries connected through a shared target in that field.
`refines` uses direct `to` and `from` edges without clique expansion,
because refinement is a vertical relation.
`refines` does not render generated footer sections by default.
It stays available for query and tide review without making every page show refinement navigation.

Waterline `to` catches the targets named by the ripple after the edit.
For `belongs`, those targets are current review neighborhoods.
For `refines`, they are current broader entries.
Frostline `to` is disabled because `to` targets are outgoing metadata on the edited ripple entry itself.
Old outgoing targets were visible where the edit happened,
so reviewing every removed target would make ordinary retargeting noisy.
Waterline and frostline `from` are both enabled because incoming neighbors live in other entries.
The editor may not have opened those dependents,
so the tide should surface both current and former entries that point at the ripple.
For `belongs`, waterline `clique` surfaces the current review neighborhood around a changed member.
Frostline clique is disabled because former peer groups usually mean a deliberate neighborhood move.

The clique semantics are the same for rendering and tide generation:
the target links to its members,
and each member links to the target and to the other members.

This policy is configuration, not *entry* data.
Changing it alters rendered navigation and review obligations
without changing structural metadata.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation](generated-navigation.md)
- belongs (from): (none)

> **Sirno generated links end.**
