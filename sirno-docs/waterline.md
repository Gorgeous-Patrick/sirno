---
name: Waterline
desc: The current public lake state, one side of the ripple comparison.
category:
  - concept
belongs:
  - ripple
prerequisite:
  - ripple
---

The *waterline* is the current public *lake*:
the editable Markdown *entries* as they exist on disk right now.
It is one of the two states a *ripple* compares.

Sirno reads the waterline from the configured lake path.
It strips each *entry*'s generated footer region before comparison,
so rendered navigation never counts as a change.
The remaining canonical metadata and prose are what the waterline contributes.

The waterline is also the live side of *wave* derivation.
When a configured structural edge enables `ripple.lake`,
Sirno reads the changed *entry*'s neighbors from the waterline
and adds them to the *tide*.
These are the neighbors that exist *after* the edit,
so a *belongs* target reached this way is a current review neighborhood.

The waterline has no stored identity.
It is whatever the public *lake* currently holds.
Freezing it with a *frost* commit turns the present waterline into the next *frostline*.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [ripple](ripple.md)
- belongs (from): (none)

> **Sirno generated links end.**
