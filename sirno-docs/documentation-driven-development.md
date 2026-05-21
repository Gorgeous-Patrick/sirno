---
name: Documentation-Driven Development
desc: Sirno's working bet that durable design lives in human-reviewed lake entries that bind future agent and human work.
category:
  - meta
  - narrative
belongs:
  - development-style
prerequisite:
  - introduction
  - methodology
  - entry-freeze
  - concept-driven-development
---

Documentation-driven development is Sirno's working bet.

This is not only document management.
It is project management:
an anti-drift discipline between the *repository* and the *lake*,
and a guard against locally reasonable designs that fall apart at project scope.
A local edit can look clean while the whole project quietly drifts toward a worse shape.
The *lake* gives that drift a name and a place to be reviewed.

In an era where agents compose code,
humans should stay in charge of the *lake*.
Review happens at the grain of one *entry*:
small enough to hold in one head,
durable enough to outlive the edit that introduced it.
When an *entry* has been read and approved,
mark it.
That mark is *entry freeze*.

A frozen *entry* is a contract.
Future agents — LLM or human alike — acknowledge it before they touch it.
To melt an *entry* is to alter it, to evolve.
That move is not free.
It demands re-understanding.
In a more familiar word, we need to learn.
The *waterline* moves,
the *frostline* reopens dependent review through *tide*,
and the project pays the cognitive cost of the change once,
deliberately,
instead of accruing it as silent drift.

Documentation-driven development also reimagines what documentation means.
Everything that carries durable meaning belongs in documentation
in an expandable and self-explanatory shape.
The Markdown body is the default,
but it is not the ceiling.
Including program, configuration, or diagram artifacts inside an *entry* is always acceptable.
When a slice of code expresses an algorithm more clearly than prose,
the code — or its core slice — belongs inside the *entry*.
A state diagram, a small grammar, a typed shape, a fixture:
choose whichever representation lowers the cognitive burden.
The *repository* still owns running code,
but the *entry* should own the version that explains it.

The contract holds because each side does its job.
Humans approve at the granularity of one *entry*.
Agents act inside what was approved.
When the design has to change,
the *lake* changes first,
the agreement is renewed,
and complexity stays managable as the project grows.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [development-style](development-style.md)
- belongs (from): (none)

> **Sirno generated links end.**
