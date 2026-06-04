---
name: Structural Link Policy
desc: Projection and Tide behavior for structural links.
category:
  - concept
belongs:
  - generated-navigation
prerequisite:
  - structural
  - generated-footer
---

Structural link policy separates relation registration, projection, and review.

| Owner | Stored form | Controls |
|---|---|---|
| relation entry | `meta.type: "structural"` | Relation registration. |
| mist spec | `[render.structural]` lists | Generated navigation in that mist. |
| relation entry | `meta.ripple.lake` and `meta.ripple.anchor` lists | Tide review obligations. |

The relation entry declares `meta.type: "structural"`.
Its entry address is the metadata field name for the relation.
Sirno orders structural relations by entry address wherever an operational order is needed.

Both owners use the same edge names:

| Edge | Meaning |
|---|---|
| `to` | Follows outgoing metadata targets. |
| `from` | Follows incoming sources that name the current *entry*. |
| `clique` | Follows entries connected through a shared target in that relation. |

A mist spec lists rendered edge directions per relation under `[render.structural]`.
That table belongs to projection settings,
so different mists can render different structural surfaces.
Relation order in the mist spec is the rendered relation order.
In relation metadata,
edge names may appear in the flat `meta.ripple.lake` and `meta.ripple.anchor` lists.
Absent render and tide values are false.

```toml
[render.structural]
belongs = ["to", "from"]
```

```yaml
meta.type: "structural"
meta.ripple.lake: ["to", "from"]
meta.ripple.anchor: ["from"]
```

`prerequisite` and `refines` use direct `to` and `from` edges without clique expansion,
because both relations are directional.
They do not render generated footer sections by default.
They stay available for query and tide review without making every page show dependency navigation.

Waterline `to` catches the targets named by the ripple after the edit.
For `belongs`, those targets are current review neighborhoods.
For `prerequisite`, they are current knowledge dependencies.
For `refines`, they are current broader entries.
Anchor-side `to` is disabled because `to` targets are outgoing metadata on the edited ripple entry itself.
Old outgoing targets were visible where the edit happened,
so reviewing every removed target would make ordinary retargeting noisy.
Waterline and Anchor-side `from` are both enabled because incoming neighbors live in other entries.
The editor may not have opened those dependents,
so the tide should surface both current and former entries that point at the ripple.
For `belongs`, waterline `clique` surfaces the current review neighborhood around a changed member.
Anchor-side clique is disabled because former peer groups usually mean a deliberate neighborhood move.

The clique semantics are shared by rendering and tide generation:
the target links to its members,
and each member links to the target and to the other members.
Mist settings choose whether a rendered projection uses clique.
Relation metadata chooses whether Tide uses clique.
