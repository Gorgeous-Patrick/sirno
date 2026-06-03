---
name: Actualize
desc: The movement from the Sirno Lake into repository material.
category:
  - concept
belongs:
  - transform
prerequisite:
  - transform
---

`actualize` moves from `lake` to `repo`.

Actualizing uses the canonical *lake* to shape repository material.
Before editing source, tests, configuration, README files, generated output,
or design documents outside the *lake*,
read the *entries* that govern the work,
follow their category, belongs, prerequisite, and refines structure,
and inspect any witnessed *repository* regions.

An actualization step should be able to answer which *entry* explains a local commitment.
Not every line needs its own *entry*,
but important commitments need a nominal place.

Actualization is where named design becomes usable material.
The *lake* should tell the implementer what matters:
which concept is being made concrete,
which field or invariant must be preserved,
and which existing *witnesses* should be inspected before editing.

The repo change should stay honest to the *lake*.
If the *entry* is still correct,
the repository material can proceed under that name.
If the repository material reveals that the *entry* is incomplete or misleading,
the work should include internalization so the *lake* learns before the material becomes canonical by habit.

Actualization is not limited to implementation.
A README section, generated manual, skill wrapper, test fixture, or design document outside the *lake*
is repo material too.
Those artifacts can be rebuilt, revised, or discarded from the maintained *lake*.

The important part is that local repository material does not float free of design intent.
Future readers should be able to ask why a piece of repo material exists
and find the *entry* that gave the commitment a name.
