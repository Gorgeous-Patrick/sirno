---
name: Structural Link Policy
desc: Rendering and Tide behavior for structural links.
category:
  - concept
belongs:
  - generated-navigation
prerequisite:
  - structural
  - generated-footer
---

Structural link policy has two owners.

| Owner | Stored form | Controls |
|---|---|---|
| `Sirno.toml` | `[structural.FIELD].entry` | Relation registration. |
| `Sirno.toml` | `[render.structural]` lists | Generated footer rendering. |
| relation entry | `meta.ripple.lake` and `meta.ripple.frost` lists | Sirno Tide review obligations. |

`Sirno.toml` also preserves relation order in `[structural.FIELD]`.
The relation entry declares `meta.type: "structural"`.

Both owners use the same edge names:

| Edge | Meaning |
|---|---|
| `to` | Follows outgoing metadata targets. |
| `from` | Follows incoming sources that name the current *entry*. |
| `clique` | Follows entries connected through a shared target in that relation. |

In `Sirno.toml`, `[render.structural]` lists rendered edge directions per relation.
In relation metadata,
edge names may appear in the flat `meta.ripple.lake` and `meta.ripple.frost` lists.
Absent render and tide values are false.

```toml
[structural.belongs]
entry = "belongs"

[structural.refines]
entry = "refines"

[structural.prerequisite]
entry = "prerequisite"

[render.structural]
belongs = ["to", "from"]
```

```yaml
meta.type: "structural"
meta.ripple.lake: ["to", "from"]
meta.ripple.frost: ["from"]
```

`prerequisite` and `refines` use direct `to` and `from` edges without clique expansion,
because both relations are directional.
They do not render generated footer sections by default.
They stay available for query and tide review without making every page show dependency navigation.

Waterline `to` catches the targets named by the ripple after the edit.
For `belongs`, those targets are current review neighborhoods.
For `prerequisite`, they are current knowledge dependencies.
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

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation](generated-navigation.md)
- belongs (from): (none)

> **Sirno generated links end.**
