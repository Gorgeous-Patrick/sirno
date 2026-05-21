---
name: Frostline
desc: The latest frost snapshot, one side of the ripple comparison.
category:
  - concept
belongs:
  - ripple
prerequisite:
  - ripple
---

The *frostline* is the latest frost snapshot:
the most recently committed frozen *lake* state.
It is the other state a *ripple* compares against the *waterline*.

Sirno reads the frostline from the configured *frost* path,
not from the lake.
The frost layer stores canonical metadata and prose without generated footers,
so the frostline already excludes rendered navigation.

The frostline is the stable side of *wave* derivation.
When a configured structural edge enables `ripple.frost`,
Sirno reads the changed *entry*'s neighbors from the frostline
and adds them to the *tide*.
These are the neighbors that existed *before* the edit.
This surfaces former dependents that the editor may never have opened,
such as a *belongs* member moved out of a neighborhood.

Before the first *frost* commit the frostline is empty.
Every lake *entry* then differs from an absent counterpart,
so the first commit can surface the whole *lake* as a bootstrap *tide*.
Each successful *frost* commit advances the frostline to the committed snapshot.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [ripple](ripple.md)
- belongs (from): (none)

> **Sirno generated links end.**
