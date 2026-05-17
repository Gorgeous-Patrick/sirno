---
name: Structural Edge Policy
desc: Configuration that chooses rendered links and tide review neighbors for structural edges.
category:
  - concept
belongs:
  - generated-navigation
  - frost-versioning
refines:
  - structural-field
---

Structural edge policy decides how configured structural edges participate in Sirno tooling.

`[structural]` lists the metadata fields Sirno treats as structural.
Each field key has `to`, `from`, and `clique` edge policies.
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
```

`to` follows outgoing metadata targets.
`from` follows incoming sources that name the current *entry*.
`clique` follows entries connected through a shared target in that field.
The clique semantics are the same for rendering and tide generation:
the target links to its members,
and each member links to the target and to the other members.

This policy is configuration, not *entry* data.
Changing it alters rendered navigation and review obligations
without changing structural metadata.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning](frost-versioning.md)
  - [generated-navigation](generated-navigation.md)
- belongs (from): (none)

> **Sirno generated links end.**
